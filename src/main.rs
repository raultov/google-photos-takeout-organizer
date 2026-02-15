mod gallery;
mod organizer;

use anyhow::{Result, bail};
use clap::Parser;
use log::{info, warn};
use std::path::Path;

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
    organizer::ui::init_logger();

    let args = Args::parse();
    let output_path = Path::new(&args.output);

    if let Some(input_str) = &args.input {
        let input_path = Path::new(input_str);
        if !input_path.exists() {
            bail!("Input path does not exist: {:?}", input_path);
        }
        organizer::organize_files(input_path, output_path, &args.unknown_dir)?;
    } else {
        info!("No input directory provided. Skipping organization.");
    }

    if args.generate_html {
        if output_path.exists() {
            gallery::generate_gallery(output_path)?;
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
