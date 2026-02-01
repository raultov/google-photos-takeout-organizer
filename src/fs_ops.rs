use anyhow::{Context, Result};
use chrono::{DateTime, Datelike, Utc};
use log::{debug, info, warn};
use std::fs;
use std::path::Path;

pub fn should_process_file(path: &Path) -> bool {
    if path.is_dir() {
        return false;
    }

    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_lowercase())
        .unwrap_or_default();

    if ext == "json" || ext.is_empty() {
        return false;
    }

    true
}

pub fn process_file(
    input_path: &Path,
    output_path: &Path,
    date: Option<DateTime<Utc>>,
    unknown_dir: &str,
) -> Result<bool> {
    let dest_folder = match date {
        Some(date) => output_path.join(format!(
            "{}/{:02}/{:02}",
            date.year(),
            date.month(),
            date.day()
        )),
        None => {
            warn!(
                "Date unknown for file: {:?}",
                input_path.file_name().unwrap_or_default()
            );
            output_path.join(unknown_dir)
        }
    };

    fs::create_dir_all(&dest_folder).context("Failed to create destination folder")?;

    if let Some(filename) = input_path.file_name() {
        let dest_path = dest_folder.join(filename);

        return if dest_path.exists() {
            let input_metadata = fs::metadata(input_path)?;
            let dest_metadata = fs::metadata(&dest_path)?;

            if input_metadata.len() != dest_metadata.len() {
                info!("Updating file (size changed): {:?}", filename);
                fs::copy(input_path, &dest_path).with_context(|| {
                    format!("Failed to copy file {:?} to {:?}", input_path, dest_path)
                })?;
                Ok(true)
            } else {
                debug!(
                    "Skipping file (already exists and same size): {:?}",
                    filename
                );
                Ok(false)
            }
        } else {
            fs::copy(input_path, &dest_path).with_context(|| {
                format!("Failed to copy file {:?} to {:?}", input_path, dest_path)
            })?;
            debug!(
                "Copied {:?} -> {:?}",
                input_path.file_name().unwrap(),
                dest_folder
            );
            Ok(true)
        };
    }

    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_process_file() {
        assert!(should_process_file(Path::new("photo.jpg")));
        assert!(should_process_file(Path::new("video.mp4")));
        assert!(should_process_file(Path::new("IMAGE.PNG")));

        // Should reject
        assert!(!should_process_file(Path::new("metadata.json")));
        assert!(!should_process_file(Path::new("no_extension")));
        assert!(!should_process_file(Path::new(".hidden")));
    }
}
