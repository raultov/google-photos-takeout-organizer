use indicatif::{ProgressBar, ProgressStyle};
use log::{LevelFilter, Metadata, Record};
use std::sync::OnceLock;

static PROGRESS_BAR: OnceLock<ProgressBar> = OnceLock::new();

struct IndicatifLogger;

impl log::Log for IndicatifLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= log::max_level()
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let msg = format!("[{}] {}", record.level(), record.args());
            if let Some(pb) = PROGRESS_BAR.get() {
                pb.println(msg);
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

    // Store the progress bar globally so the logger can use it.
    // OnceLock::set returns an Error if the value is already set.
    // We ignore this error because in tests this function might be called multiple times,
    // and in production we only care that *a* progress bar is set.
    let _ = PROGRESS_BAR.set(progress_bar.clone());

    progress_bar
}

#[cfg(test)]
mod tests {
    use super::*;
    use log::Log;

    #[test]
    fn test_create_progress_bar() {
        let total = 100;
        let pb = create_progress_bar(total);
        assert_eq!(pb.length(), Some(total));
    }

    #[test]
    fn test_logger_enabled() {
        // This test assumes default log level might be INFO or similar.
        // We can't easily reset the global logger, so we test the struct directly.
        let logger = IndicatifLogger;
        let metadata = Metadata::builder().level(log::Level::Error).build();

        // We need to ensure max_level is at least Error for this to be true.
        log::set_max_level(LevelFilter::Error);
        assert!(logger.enabled(&metadata));
    }
}
