pub mod date_utils;
pub mod fs_ops;
pub mod metadata;
pub mod model;
pub mod ui;

use anyhow::Result;
use log::{error, info};
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

use self::fs_ops::FileAction;
use self::metadata::DateExtractor;

pub fn organize_files(input_path: &Path, output_path: &Path, unknown_dir: &str) -> Result<()> {
    info!("Starting organization...");
    info!("Source: {:?}", input_path);
    info!("Dest:   {:?}", output_path);
    info!("Dir for unknown files: {:?}", unknown_dir);

    // Check if output directory is already populated (incremental run)
    let is_incremental_run = output_path.exists()
        && fs::read_dir(output_path)
            .map(|mut i| i.next().is_some())
            .unwrap_or(false);

    // First pass: Count files to initialize progress bar
    let total_files = get_total_files(input_path);
    info!("Found {} files to process", total_files);

    let progress_bar = ui::create_progress_bar(total_files);
    progress_bar.set_message("Organizing Photos:");

    let date_extractor = DateExtractor::new()?;
    let mut success_count = 0;
    let mut error_count = 0;
    let mut skipped_count = 0;
    let mut new_files = Vec::new();

    for entry in WalkDir::new(input_path).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();

        if !fs_ops::should_process_file(path) {
            continue;
        }

        let date = date_extractor.determine_date(path);

        match fs_ops::process_file(path, output_path, date, unknown_dir) {
            Ok(action) => match action {
                FileAction::New => {
                    success_count += 1;
                    if let Some(name) = path.file_name() {
                        new_files.push(name.to_string_lossy().to_string());
                    }
                }
                FileAction::Updated => {
                    success_count += 1;
                }
                FileAction::Skipped => {
                    skipped_count += 1;
                }
            },
            Err(e) => {
                error!("Failed to process {:?}: {}", path, e);
                error_count += 1;
            }
        }
        progress_bar.inc(1);
    }

    progress_bar.finish_with_message("Done");

    info!(
        "Done! Processed: {}, Skipped: {}, Errors: {}",
        success_count, skipped_count, error_count
    );

    if is_incremental_run {
        if !new_files.is_empty() {
            info!("--- New Files Detected ---");
            for file in new_files {
                info!("  - {}", file);
            }
            info!("--------------------------");
        } else {
            info!("No new file found!");
        }
    }

    Ok(())
}

fn get_total_files(input_path: &Path) -> u64 {
    WalkDir::new(input_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| fs_ops::should_process_file(e.path()))
        .count() as u64
}
