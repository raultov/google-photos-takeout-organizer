use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use sysinfo::System;

pub struct ThrottleState {
    pub active: usize,
    pub limit: usize,
    pub last_increase: Instant,
}

#[derive(Clone)]
pub struct Throttle {
    state: Arc<Mutex<ThrottleState>>,
    increase_interval: Duration,
    check_interval: Duration,
}

impl Throttle {
    /// Creates a new Throttle instance.
    /// `increase_interval` is the amount of time to wait before allowing another thread if memory is good.
    /// `check_interval` is the sleep duration between memory checks when waiting for a slot.
    pub fn new(increase_interval: Duration, check_interval: Duration) -> Self {
        Self {
            state: Arc::new(Mutex::new(ThrottleState {
                active: 0,
                // Start with 1 thread minimum
                limit: 1,
                last_increase: Instant::now(),
            })),
            increase_interval,
            check_interval,
        }
    }

    /// Blocks until it's safe to start a new job based on available memory and current active jobs.
    /// Returns (active_count, percent_available, limit, is_fallback).
    pub fn wait_for_slot(&self, sys: &mut System) -> (usize, u64, usize, bool) {
        loop {
            sys.refresh_memory();
            let available_mem = sys.available_memory();
            let total_mem = sys.total_memory();
            let percent_available = (available_mem as f64 / total_mem as f64 * 100.0) as u64;

            let mut should_wait = true;
            let mut current_active = 0;
            let mut current_limit = 0;

            if let Ok(mut state) = self.state.lock() {
                current_active = state.active;

                // Gradual up-scaling: If free memory is >= 30% and we've waited enough time
                // since the last increase, bump the concurrency limit up by 1.
                if percent_available >= 30 {
                    if state.last_increase.elapsed() >= self.increase_interval {
                        state.limit += 1;
                        state.last_increase = Instant::now();
                    }
                } else if percent_available < 20 && state.limit > 1 {
                    // Downward pressure if memory is running short
                    state.limit -= 1;
                }

                current_limit = state.limit;

                // Can we start a new job now?
                if state.active < state.limit {
                    state.active += 1;
                    should_wait = false;
                }
            }

            if !should_wait {
                return (current_active + 1, percent_available, current_limit, false);
            }

            // Log minimal fallback wait if active == 0 to avoid sticking forever
            if current_active == 0
                && let Ok(mut state) = self.state.lock()
                && state.active == 0
            {
                state.active += 1;
                return (1, percent_available, current_limit, true);
            }

            std::thread::sleep(self.check_interval);
        }
    }

    /// Decrements the active thread count.
    /// Returns (active_now, current_limit).
    pub fn release_slot(&self) -> (usize, usize) {
        let mut active_now = 0;
        let mut current_limit = 0;
        if let Ok(mut state) = self.state.lock() {
            if state.active > 0 {
                state.active -= 1;
            }
            active_now = state.active;
            current_limit = state.limit;
        }
        (active_now, current_limit)
    }

    /// Read-only snapshot of limit and active.
    #[allow(dead_code)]
    pub fn get_stats(&self) -> (usize, usize) {
        if let Ok(state) = self.state.lock() {
            (state.active, state.limit)
        } else {
            (0, 0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rayon::prelude::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn test_parallel_transcoding_logic() {
        // Simulate 5 videos to transcode
        let videos = vec![1, 2, 3, 4, 5];

        let throttle = Throttle::new(Duration::from_millis(50), Duration::from_millis(10));
        let max_concurrent = Arc::new(AtomicUsize::new(0));

        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(4)
            .build()
            .unwrap();

        pool.install(|| {
            videos.into_par_iter().for_each(|_| {
                let mut sys = sysinfo::System::new();

                let (active, _, _, _) = throttle.wait_for_slot(&mut sys);

                // Track the max concurrent threads observed
                let mut current_max = max_concurrent.load(Ordering::SeqCst);
                while active > current_max {
                    let result = max_concurrent.compare_exchange(
                        current_max,
                        active,
                        Ordering::SeqCst,
                        Ordering::SeqCst,
                    );
                    match result {
                        Ok(_) => break,
                        Err(actual) => current_max = actual,
                    }
                }

                // Simulate Transcoding work. It needs to be longer than the 50ms
                // scale-up tick so that concurrent threads can actually overlap.
                std::thread::sleep(std::time::Duration::from_millis(150));

                throttle.release_slot();
            });
        });

        // We assert that if our system has >30% free memory when running the test,
        // it must have processed more than 1 video concurrently.
        let mut sys = sysinfo::System::new();
        sys.refresh_memory();
        if sys.available_memory() * 100 >= sys.total_memory() * 30 {
            // Can be slightly flaky on heavily loaded CI runners, but locally on typical
            // multi-core development machines with free memory, Rayon will spawn >1 task.
            if num_cpus::get() > 1 {
                assert!(
                    max_concurrent.load(Ordering::SeqCst) > 1,
                    "Expected >1 parallel transcode tasks with sufficient memory"
                );
            }
        } else {
            assert!(
                max_concurrent.load(Ordering::SeqCst) >= 1,
                "Expected at least 1 fallback task on low memory"
            );
        }
    }
}
