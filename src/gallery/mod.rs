mod builder;
mod media;
mod utils;

use anyhow::{Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use log::{info, warn};
use std::fs;
use std::path::Path;
use std::sync::Mutex;

pub use self::builder::build_html;
pub use self::utils::{is_image, is_video};

pub fn generate_gallery(root_path: &Path, threads: usize, transcode_videos: bool) -> Result<()> {
    info!("Generating HTML gallery in {:?}", root_path);

    let has_ffmpeg = {
        let available = media::check_ffmpeg_available();
        if !available {
            warn!(
                "ffmpeg not found. Video thumbnails will not be generated. Please install ffmpeg."
            );
        }
        available
    };

    info!(
        "Generating thumbnails in parallel (threads: {}, transcoding: {})",
        threads, transcode_videos
    );

    // Quick count for progress bar (shallow scan)
    let total_files = walkdir::WalkDir::new(root_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| is_image(e.path()) || is_video(e.path()))
        .count();

    let pb = ProgressBar::new(total_files as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("Processing Media: {spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")?
            .progress_chars("#>-"),
    );

    // Process using a parallel iterator
    use rayon::prelude::*;

    // Collect paths
    let media_paths: Vec<_> = walkdir::WalkDir::new(root_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| is_image(e.path()) || is_video(e.path()))
        .map(|e| e.path().to_path_buf())
        .collect();

    // 1. Parallel Phase: Generate thumbnails (Fast)
    // Collect videos that need transcoding to process them sequentially later
    let to_transcode = Mutex::new(Vec::new());

    media_paths.into_par_iter().for_each(|path| {
        match ensure_thumbnail_fast(&path, has_ffmpeg) {
            Ok(needs_transcode) => {
                if needs_transcode && transcode_videos && let Ok(mut list) = to_transcode.lock() {
                    list.push(path);
                }
            }
            Err(e) => warn!("Failed to process {:?}: {}", path, e),
        }
        pb.inc(1);
    });
    pb.finish_with_message("Thumbnails Generated");

    // 2. Sequential Phase: Transcode videos (Heavy)
    if transcode_videos {
        let videos = to_transcode.into_inner().unwrap_or_default();
        if !videos.is_empty() {
            let tv_pb = ProgressBar::new(videos.len() as u64);
            tv_pb.set_style(
                ProgressStyle::default_bar()
                    .template("Transcoding Videos: {spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")?
                    .progress_chars("#>-"),
            );

            for path in videos {
                if let Err(e) = transcode_video_sequential(&path) {
                    warn!("Failed to transcode {:?}: {}", path, e);
                }
                tv_pb.inc(1);
            }
            tv_pb.finish_with_message("Transcoding Completed");
        }
    }

    let pb_html = ProgressBar::new(total_files as u64);
    pb_html.set_style(
        ProgressStyle::default_bar()
            .template("Generating HTML: {spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")?
            .progress_chars("#>-"),
    );

    visit_dir(root_path, root_path, &pb_html)?;
    pb_html.finish_with_message("Gallery Generated");
    Ok(())
}

fn visit_dir(dir: &Path, root_path: &Path, pb: &ProgressBar) -> Result<()> {
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
            media_files.push(path);
            pb.inc(1);
        }
    }

    subdirs.sort();
    media_files.sort();

    // Custom sort for months if we are at the year level
    let relative_path = dir.strip_prefix(root_path).unwrap_or(Path::new(""));
    let depth = relative_path.components().count();

    if depth == 1 {
        subdirs.sort_by_key(|p| {
            let month_name = p.file_name().unwrap_or_default().to_string_lossy();
            match month_name.as_ref() {
                "January" => 1,
                "February" => 2,
                "March" => 3,
                "April" => 4,
                "May" => 5,
                "June" => 6,
                "July" => 7,
                "August" => 8,
                "September" => 9,
                "October" => 10,
                "November" => 11,
                "December" => 12,
                _ => 13,
            }
        });
    }

    // Recurse first
    for subdir in &subdirs {
        visit_dir(subdir, root_path, pb)?;
    }

    // Check if we should generate a flattened view (Depth 2 = Month level)
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

fn ensure_thumbnail_fast(media_path: &Path, has_ffmpeg: bool) -> Result<bool> {
    let parent = media_path.parent().unwrap();
    let thumb_dir = parent.join(".thumbnails");
    let filename = media_path.file_name().unwrap().to_string_lossy();
    let thumb_name = format!("{}.jpg", filename);
    let thumb_path = thumb_dir.join(&thumb_name);

    if !thumb_dir.exists() {
        let _ = fs::create_dir_all(&thumb_dir);
    }

    if is_image(media_path) {
        if !thumb_path.exists() {
            media::generate_image_thumbnail(media_path, &thumb_path)?;
        }
        Ok(false)
    } else if is_video(media_path) {
        if !has_ffmpeg {
            return Ok(false);
        }

        // Generate thumbnail if needed (Fast)
        if !thumb_path.exists() {
            let duration = media::get_video_duration(media_path).unwrap_or(0.0);
            let time_pos = if duration > 0.0 { duration / 2.0 } else { 0.0 };
            media::generate_thumbnail(media_path, &thumb_path, time_pos)?;
        }

        // Check if transcoding is needed
        let compatible_name = format!("{}.mp4", filename);
        let compatible_path = thumb_dir.join(&compatible_name);

        if compatible_path.exists() {
            return Ok(false);
        }

        if let Ok(codec) = media::get_video_codec(media_path) && codec == "hevc" {
            return Ok(true);
        }
        Ok(false)
    } else {
        Ok(false)
    }
}

fn transcode_video_sequential(media_path: &Path) -> Result<()> {
    let parent = media_path.parent().unwrap();
    let thumb_dir = parent.join(".thumbnails");
    let filename = media_path.file_name().unwrap().to_string_lossy();
    let compatible_name = format!("{}.mp4", filename);
    let compatible_path = thumb_dir.join(&compatible_name);

    info!("Transcoding HEVC video (SEQUENTIAL): {:?}", filename);
    media::transcode_video_to_h264(media_path, &compatible_path)
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
        let result = generate_gallery(root, 1, false);
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

    #[test]
    fn test_ensure_thumbnail_fast() {
        let root = Path::new("test_thumb_fast");
        if root.exists() {
            fs::remove_dir_all(root).unwrap();
        }
        fs::create_dir_all(root).unwrap();

        let img_path = root.join("test.jpg");
        fs::write(&img_path, "dummy content").unwrap();

        // Should return false (not HEVC).
        // We ignore the error from image library since we are testing path logic
        let _ = ensure_thumbnail_fast(&img_path, false);

        // Check if .thumbnails was created
        assert!(root.join(".thumbnails").exists());

        if root.exists() {
            fs::remove_dir_all(root).unwrap();
        }
    }
}
