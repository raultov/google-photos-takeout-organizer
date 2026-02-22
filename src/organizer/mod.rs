pub mod date_utils;
pub mod fs_ops;
pub mod metadata;
pub mod model;
pub mod ui;

use anyhow::Result;
use log::{error, info, warn};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;
use walkdir::WalkDir;

use self::fs_ops::FileAction;
use self::metadata::DateExtractor;

pub fn organize_files(input_paths: &[&Path], output_path: &Path, unknown_dir: &str) -> Result<()> {
    info!("Starting organization...");
    info!("Sources: {:?}", input_paths);
    info!("Dest:   {:?}", output_path);
    info!("Dir for unknown files: {:?}", unknown_dir);
    
    let mut processed_input_paths: Vec<PathBuf> = Vec::new();

    let mut archives = Vec::new();
    for input_path in input_paths {
        if fs_ops::is_archive(input_path) {
            archives.push(input_path.to_path_buf());
        } else if input_path.is_dir() {
            processed_input_paths.push(input_path.to_path_buf());
            // Scan directory for internal archives too
            for entry in fs::read_dir(input_path)? {
                let entry = entry?;
                let path = entry.path();
                if fs_ops::is_archive(&path) {
                    archives.push(path);
                }
            }
        }
    }

    let temp_dir: Option<TempDir> = if !archives.is_empty() {
        let temp = TempDir::new()?;
        for archive in archives {
            if let Err(e) = fs_ops::extract_archive(&archive, temp.path()) {
                warn!("Failed to extract archive {:?}: {}", archive, e);
            }
        }
        processed_input_paths.push(temp.path().to_path_buf());
        Some(temp)
    } else {
        None
    };

    // Check if output directory is already populated (incremental run)
    let is_incremental_run = output_path.exists()
        && fs::read_dir(output_path)
            .map(|mut i| i.next().is_some())
            .unwrap_or(false);

    // First pass: Count files to initialize progress bar
    let mut total_files = 0;
    for path in &processed_input_paths {
        total_files += get_total_files(path);
    }
    info!("Found {} files to process", total_files);

    let progress_bar = ui::create_progress_bar(total_files);
    progress_bar.set_message("Organizing Photos:");

    let date_extractor = DateExtractor::new()?;
    let mut success_count = 0;
    let mut error_count = 0;
    let mut skipped_count = 0;
    let mut new_files = Vec::new();

    for source_path in &processed_input_paths {
        for entry in WalkDir::new(source_path).into_iter().filter_map(|e| e.ok()) {
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

    // If we used a temporary directory, it will be deleted when temp_dir is dropped
    if temp_dir.is_some() {
        info!("Cleaning up temporary directory...");
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
