mod builder;
mod images;
mod media;
mod traversal;
mod utils;
mod videos;

use anyhow::Result;
use indicatif::ProgressBar;
use log::{info, warn};
use std::path::Path;
use std::sync::Mutex;

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

    // Collect paths once to avoid redundant filesystem traversal
    let media_paths: Vec<_> = walkdir::WalkDir::new(root_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| is_image(e.path()) || is_video(e.path()))
        .map(|e| e.path().to_path_buf())
        .collect();

    let total_files = media_paths.len();

    use indicatif::{MultiProgress, ProgressStyle};
    let multi = MultiProgress::new();

    let pb_style = ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({msg})")?
        .progress_chars("#>-");

    let pb = multi.add(ProgressBar::new(total_files as u64));
    pb.set_style(pb_style.clone());
    pb.set_message("Processing Media");
    crate::organizer::ui::set_global_progress_bar(pb.clone());

    // Process using a parallel iterator
    use rayon::prelude::*;

    // 1. Parallel Phase: Generate thumbnails (Fast)
    // Collect videos that need transcoding
    let to_transcode = Mutex::new(Vec::new());

    media_paths.into_par_iter().for_each(|path| {
        match videos::ensure_thumbnail_fast(&path, has_ffmpeg) {
            Ok(needs_transcode) => {
                if needs_transcode
                    && transcode_videos
                    && let Ok(mut list) = to_transcode.lock()
                {
                    list.push(path);
                }
            }
            Err(e) => warn!("Failed to process {:?}: {}", path, e),
        }
        pb.inc(1);
    });
    pb.finish_with_message("Thumbnails Done");

    // 2. Phase: Transcode videos (Heavy)
    if transcode_videos {
        let videos = to_transcode.into_inner().unwrap_or_default();
        if !videos.is_empty() {
            let tv_pb = multi.add(ProgressBar::new(videos.len() as u64));
            tv_pb.set_style(pb_style.clone());
            tv_pb.set_message("Transcoding Videos Sequentially");
            crate::organizer::ui::set_global_progress_bar(tv_pb.clone());

            // Process sequentially to avoid OOM
            for path in videos {
                if let Err(e) = videos::transcode_sequential(&path) {
                    warn!("Failed to transcode {:?}: {}", path, e);
                }
                tv_pb.inc(1);
            }
            tv_pb.finish_with_message("Transcoding Done");
        }
    }

    let pb_html = multi.add(ProgressBar::new(total_files as u64));
    pb_html.set_style(pb_style);
    pb_html.set_message("Generating HTML");
    crate::organizer::ui::set_global_progress_bar(pb_html.clone());

    traversal::visit_dir(root_path, root_path, &pb_html)?;
    pb_html.finish_with_message("Gallery Done");
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
        let _ = videos::ensure_thumbnail_fast(&img_path, false);

        // Check if .thumbnails was created
        assert!(root.join(".thumbnails").exists());

        if root.exists() {
            fs::remove_dir_all(root).unwrap();
        }
    }
}
