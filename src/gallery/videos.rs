use super::images;
use super::media;
use super::utils::{is_image, is_video};
use anyhow::Result;
use log::debug;
use std::fs;
use std::path::Path;

/// Dispatcher to process media fast (thumbnails and transcode check).
pub fn ensure_thumbnail_fast(media_path: &Path, has_ffmpeg: bool) -> Result<bool> {
    if is_image(media_path) {
        images::ensure_thumbnail(media_path)?;
        Ok(false)
    } else if is_video(media_path) {
        ensure_thumbnail_and_check_transcode(media_path, has_ffmpeg)
    } else {
        Ok(false)
    }
}

/// Ensures a thumbnail exists for the given video and returns true if it needs transcoding.
pub fn ensure_thumbnail_and_check_transcode(video_path: &Path, has_ffmpeg: bool) -> Result<bool> {
    let parent = video_path.parent().unwrap();
    let thumb_dir = parent.join(".thumbnails");
    let filename = video_path.file_name().unwrap().to_string_lossy();
    let thumb_path = thumb_dir.join(format!("{}.jpg", filename));

    if !thumb_dir.exists() {
        let _ = fs::create_dir_all(&thumb_dir);
    }

    if !has_ffmpeg {
        return Ok(false);
    }

    // Generate thumbnail if needed (Fast)
    if !thumb_path.exists() {
        let duration = media::get_video_duration(video_path).unwrap_or(0.0);
        let time_pos = if duration > 0.0 { duration / 2.0 } else { 0.0 };
        media::generate_thumbnail(video_path, &thumb_path, time_pos)?;
    }

    // Check if transcoding is needed
    let compatible_path = thumb_dir.join(format!("{}.mp4", filename));

    if compatible_path.exists() {
        return Ok(false);
    }

    if let Ok(codec) = media::get_video_codec(video_path)
        && codec == "hevc"
    {
        return Ok(true);
    }
    Ok(false)
}

/// Transcodes a video to a web-compatible format sequentially.
pub fn transcode_sequential(video_path: &Path) -> Result<()> {
    let parent = video_path.parent().unwrap();
    let thumb_dir = parent.join(".thumbnails");
    let filename = video_path.file_name().unwrap().to_string_lossy();
    let compatible_path = thumb_dir.join(format!("{}.mp4", filename));

    debug!("Transcoding video (Sequential): {:?}", filename);
    media::transcode_video_to_h264(video_path, &compatible_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_ensure_thumbnail_fast_dispatch() {
        let dir = tempdir().unwrap();
        let img_path = dir.path().join("test.jpg");
        let vid_path = dir.path().join("test.mp4");

        // Images should try to thumbnail (and fail here because file is missing)
        let result_img = ensure_thumbnail_fast(&img_path, false);
        assert!(result_img.is_err());

        // Videos with has_ffmpeg=false should return false immediately after dir creation
        let result_vid = ensure_thumbnail_fast(&vid_path, false);
        assert!(result_vid.is_ok());
        assert!(!result_vid.unwrap());
    }

    #[test]
    fn test_ensure_thumbnail_and_check_transcode_no_ffmpeg() {
        let dir = tempdir().unwrap();
        let vid_path = dir.path().join("test.mp4");

        let needs_transcode = ensure_thumbnail_and_check_transcode(&vid_path, false).unwrap();
        assert!(!needs_transcode);
        assert!(dir.path().join(".thumbnails").exists());
    }
}
