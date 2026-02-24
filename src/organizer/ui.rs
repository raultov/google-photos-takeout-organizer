use indicatif::{ProgressBar, ProgressStyle};
use log::{LevelFilter, Metadata, Record};
use std::sync::Mutex;

static PROGRESS_BAR: Mutex<Option<ProgressBar>> = Mutex::new(None);

struct IndicatifLogger;

impl log::Log for IndicatifLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= log::max_level()
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let msg = format!("[{}] {}", record.level(), record.args());
            if let Ok(pb_opt) = PROGRESS_BAR.lock() {
                if let Some(pb) = &*pb_opt {
                    pb.println(msg);
                } else {
                    eprintln!("{}", msg);
                }
            } else {
                eprintln!("{}", msg);
            }
        }
    }

    fn flush(&self) {}
}

static LOGGER: IndicatifLogger = IndicatifLogger;

pub fn init_logger() {
    let level = std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());
    let level_filter = match level.to_lowercase().as_str() {
        "error" => LevelFilter::Error,
        "warn" => LevelFilter::Warn,
        "debug" => LevelFilter::Debug,
        "trace" => LevelFilter::Trace,
        _ => LevelFilter::Info,
    };

    // set_logger can only be called once. We ignore the error if it's already set
    // to allow tests to run without panicking.
    let _ = log::set_logger(&LOGGER);
    log::set_max_level(level_filter);
}

pub fn set_global_progress_bar(pb: ProgressBar) {
    if let Ok(mut pb_opt) = PROGRESS_BAR.lock() {
        *pb_opt = Some(pb);
    }
}

pub fn create_progress_bar(total_files: u64) -> ProgressBar {
    let progress_bar = ProgressBar::new(total_files);
    progress_bar.set_style(
        ProgressStyle::default_bar()
            .template(
                "{msg} {spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})",
            )
            .expect("Invalid progress bar template")
            .progress_chars("#>-"),
    );

    set_global_progress_bar(progress_bar.clone());

    progress_bar
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_progress_bar() {
        let total = 100;
        let pb = create_progress_bar(total);
        assert_eq!(pb.length(), Some(total));
    }
}
