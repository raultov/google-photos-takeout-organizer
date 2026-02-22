mod gallery;
mod organizer;

use anyhow::Result;
use clap::Parser;
use log::{info, warn};
use std::path::Path;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the source directory or archive files (Google Takeout). Multiple inputs allowed.
    #[arg(short, long, num_args = 1..)]
    input: Vec<String>,

    /// Path to the destination directory (Required)
    #[arg(short, long)]
    output: String,

    /// Name of the folder for files with no date
    #[arg(short, long, default_value = "unknown")]
    unknown_dir: String,

    /// Generate an HTML gallery of the organized photos
    #[arg(short, long, default_value_t = true, action = clap::ArgAction::Set)]
    generate_html: bool,

    /// Transcode HEVC videos to H.264 for better web compatibility (Heavy operation, processed sequentially)
    #[arg(short, long, default_value_t = false)]
    transcode_videos: bool,

    /// Number of parallel thumbnail generation tasks (Default: Total cores - 1)
    #[arg(short = 'j', long, default_value_t = (num_cpus::get() - 1).max(1))]
    threads: usize,
}

fn main() -> Result<()> {
    organizer::ui::init_logger();

    let args = Args::parse();

    // Configure Rayon thread pool globally
    rayon::ThreadPoolBuilder::new()
        .num_threads(args.threads)
        .build_global()
        .unwrap();

    let output_path = Path::new(&args.output);

    if !args.input.is_empty() {
        let mut input_paths = Vec::new();
        for input_str in &args.input {
            let p = Path::new(input_str);
            if !p.exists() {
                warn!("Input path does not exist, skipping: {:?}", p);
                continue;
            }
            input_paths.push(p);
        }

        if !input_paths.is_empty() {
            organizer::organize_files(&input_paths, output_path, &args.unknown_dir)?;
        }
    } else {
        info!("No input provided. Skipping organization.");
    }

    if args.generate_html {
        if output_path.exists() {
            gallery::generate_gallery(output_path, args.threads, args.transcode_videos)?;
        } else if args.input.is_empty() {
            warn!(
                "Output directory {:?} does not exist. Cannot generate HTML.",
                output_path
            );
        }
    } else if args.input.is_empty() {
        info!("No input provided and HTML generation disabled. Nothing to do.");
    }

    Ok(())
}
