mod date_utils;
mod fs_ops;
mod html;
mod metadata;
mod model;
mod ui;

use anyhow::{Result, bail};
use clap::Parser;
use log::{error, info, warn};
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

use crate::fs_ops::FileAction;
use crate::metadata::DateExtractor;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the source directory (Google Takeout). Optional if regenerating HTML.
    #[arg(short, long)]
    input: Option<String>,

    /// Path to the destination directory (Required)
    #[arg(short, long)]
    output: String,

    /// Name of the folder for files with no date
    #[arg(short, long, default_value = "unknown")]
    unknown_dir: String,

    /// Generate an HTML gallery of the organized photos
    #[arg(short, long, default_value_t = true, action = clap::ArgAction::Set)]
    generate_html: bool,
}

fn main() -> Result<()> {
    ui::init_logger();

    let args = Args::parse();
    let output_path = Path::new(&args.output);

    if let Some(input_str) = &args.input {
        let input_path = Path::new(input_str);
        if !input_path.exists() {
            bail!("Input path does not exist: {:?}", input_path);
        }
        organize_files(input_path, output_path, &args.unknown_dir)?;
    } else {
        info!("No input directory provided. Skipping organization.");
    }

    if args.generate_html {
        if output_path.exists() {
            html::generate_gallery(output_path)?;
        } else if args.input.is_none() {
            warn!(
                "Output directory {:?} does not exist. Cannot generate HTML.",
                output_path
            );
        }
    } else if args.input.is_none() {
        info!("No input provided and HTML generation disabled. Nothing to do.");
    }

    Ok(())
}

fn organize_files(input_path: &Path, output_path: &Path, unknown_dir: &str) -> Result<()> {
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
