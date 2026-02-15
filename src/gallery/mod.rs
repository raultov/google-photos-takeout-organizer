mod builder;
mod media;
mod utils;

use anyhow::{Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use log::{info, warn};
use std::fs;
use std::path::Path;

pub use self::builder::build_html;
pub use self::utils::{is_image, is_video};

pub fn generate_gallery(root_path: &Path) -> Result<()> {
    info!("Generating HTML gallery in {:?}", root_path);

    let has_ffmpeg = media::check_ffmpeg_available();
    if !has_ffmpeg {
        warn!("ffmpeg not found. Video thumbnails will not be generated. Please install ffmpeg.");
    }

    let total_files = count_media_files(root_path);
    let pb = ProgressBar::new(total_files);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("Generating Gallery: {spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")?
            .progress_chars("#>-"),
    );

    visit_dir(root_path, root_path, &pb, has_ffmpeg)?;
    pb.finish_with_message("Gallery Generated");
    Ok(())
}

fn count_media_files(dir: &Path) -> u64 {
    let mut count = 0;
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if !path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .starts_with('.')
                {
                    count += count_media_files(&path);
                }
            } else if is_image(&path) || is_video(&path) {
                count += 1;
            }
        }
    }
    count
}

fn visit_dir(dir: &Path, root_path: &Path, pb: &ProgressBar, has_ffmpeg: bool) -> Result<()> {
    if !dir.is_dir() {
        return Ok(());
    }

    let entries = fs::read_dir(dir)?;
    let mut subdirs = Vec::new();
    let mut media_files = Vec::new();

    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            // Ignore hidden directories like .thumbnails
            if !path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .starts_with('.')
            {
                subdirs.push(path);
            }
        } else if is_image(&path) || is_video(&path) {
            if is_video(&path)
                && let Err(e) = ensure_thumbnail(&path, has_ffmpeg)
            {
                warn!("Failed to generate thumbnail for {:?}: {}", path, e);
            }
            media_files.push(path);
            pb.inc(1);
        }
    }

    subdirs.sort();
    media_files.sort();

    // Recurse first
    for subdir in &subdirs {
        visit_dir(subdir, root_path, pb, has_ffmpeg)?;
    }

    // Check if we should generate a flattened view (Depth 2 = Month level)
    let relative_path = dir.strip_prefix(root_path).unwrap_or(Path::new(""));
    let depth = relative_path.components().count();
    let mut flattened_media = Vec::new();

    // Heuristic: If we are at Month level (depth 2) and have subdirectories (Days),
    // collect all media from those subdirectories to show a flattened view.
    if depth == 2 && !subdirs.is_empty() {
        for subdir in &subdirs {
            if let Ok(entries) = fs::read_dir(subdir) {
                let mut dir_media = Vec::new();
                for entry in entries.flatten() {
                    let path = entry.path();
                    if !path.is_dir() && (is_image(&path) || is_video(&path)) {
                        // Get path relative to current 'dir' (Month dir)
                        if let Ok(rel) = path.strip_prefix(dir) {
                            dir_media.push(rel.to_path_buf());
                        }
                    }
                }
                dir_media.sort();
                flattened_media.extend(dir_media);
            }
        }
    }

    // Generate HTML for current dir
    // Only generate if there are contents, or it's the root
    if !subdirs.is_empty() || !media_files.is_empty() || dir == root_path {
        let content = build_html(dir, root_path, &subdirs, &media_files, &flattened_media)?;

        let filename = if dir == root_path {
            "collection.html"
        } else {
            "index.html"
        };
        let output_file = dir.join(filename);
        fs::write(&output_file, content).context("Failed to write HTML file")?;
    }

    Ok(())
}

fn ensure_thumbnail(video_path: &Path, has_ffmpeg: bool) -> Result<()> {
    let parent = video_path.parent().unwrap();
    let thumb_dir = parent.join(".thumbnails");
    let filename = video_path.file_name().unwrap().to_string_lossy();
    let thumb_name = format!("{}.jpg", filename);
    let thumb_path = thumb_dir.join(&thumb_name);

    if thumb_path.exists() {
        return Ok(());
    }

    if !has_ffmpeg {
        return Ok(());
    }

    // Attempt to get duration to pick middle frame
    let duration = media::get_video_duration(video_path).unwrap_or(0.0);
    let time_pos = if duration > 0.0 { duration / 2.0 } else { 0.0 };

    media::generate_thumbnail(video_path, &thumb_path, time_pos)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_generate_gallery_integration() {
        let root = Path::new("test_gallery_gen");
        if root.exists() {
            fs::remove_dir_all(root).unwrap();
        }
        fs::create_dir_all(root.join("2020/01/01")).unwrap();

        // Create dummy images
        fs::write(root.join("2020/01/01/img1.jpg"), "").unwrap();
        fs::write(root.join("2020/01/01/img2.jpg"), "").unwrap();

        // Run generation
        let result = generate_gallery(root);
        assert!(result.is_ok());

        // Check root HTML
        let collection = root.join("collection.html");
        assert!(collection.exists());
        let content = fs::read_to_string(collection).unwrap();
        assert!(content.contains("2020"));

        // Check Year HTML
        let year_html = root.join("2020/index.html");
        assert!(year_html.exists());
        let content = fs::read_to_string(year_html).unwrap();
        assert!(content.contains("01"));

        // Check Month HTML (should have toggle and flattened images)
        let month_html = root.join("2020/01/index.html");
        assert!(month_html.exists());
        let content = fs::read_to_string(month_html).unwrap();
        assert!(content.contains("toggle-btn"));
        assert!(content.contains("img1.jpg"));

        // Check Day HTML
        let day_html = root.join("2020/01/01/index.html");
        assert!(day_html.exists());
        let content = fs::read_to_string(day_html).unwrap();
        assert!(content.contains("img1.jpg"));

        // Cleanup
        fs::remove_dir_all(root).unwrap();
    }
}
