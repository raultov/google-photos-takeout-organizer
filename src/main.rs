mod date_utils;
mod fs_ops;
mod metadata;
mod model;

use anyhow::{Result, bail};
use clap::Parser;
use log::{error, info};
use std::io::Write;
use std::path::Path;
use walkdir::WalkDir;

use crate::metadata::DateExtractor;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    input: String,

    #[arg(short, long)]
    output: String,

    #[arg(short, long, default_value = "unknown")]
    unknown_dir: String,
}

fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format(|buf, record| writeln!(buf, "[{}] {}", record.level(), record.args()))
        .init();

    let args = Args::parse();
    let input_path = Path::new(&args.input);
    let output_path = Path::new(&args.output);

    if !input_path.exists() {
        bail!("Input path does not exist: {:?}", input_path);
    }

    info!("Starting organization...");
    info!("Source: {:?}", input_path);
    info!("Dest:   {:?}", output_path);
    info!("Dir for unknown files: {:?}", args.unknown_dir);

    let date_extractor = DateExtractor::new()?;
    let mut success_count = 0;
    let mut error_count = 0;
    let mut skipped_count = 0;

    // TODO add progress indicator
    for entry in WalkDir::new(input_path).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();

        if !fs_ops::should_process_file(path) {
            continue;
        }

        let date = date_extractor.determine_date(path);

        match fs_ops::process_file(path, output_path, date, &args.unknown_dir) {
            Ok(processed) => {
                if processed {
                    success_count += 1;
                } else {
                    skipped_count += 1;
                }
            }
            Err(e) => {
                error!("Failed to process {:?}: {}", path, e);
                error_count += 1;
            }
        }
    }

    info!(
        "Done! Processed: {}, Skipped: {}, Errors: {}",
        success_count, skipped_count, error_count
    );
    Ok(())
}
