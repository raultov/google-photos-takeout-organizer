use super::media;
use anyhow::Result;
use std::fs;
use std::path::Path;

/// Ensures a thumbnail exists for the given image.
pub fn ensure_thumbnail(image_path: &Path) -> Result<()> {
    let parent = image_path.parent().unwrap();
    let thumb_dir = parent.join(".thumbnails");
    let filename = image_path.file_name().unwrap().to_string_lossy();
    let thumb_path = thumb_dir.join(format!("{}.jpg", filename));

    if !thumb_dir.exists() {
        let _ = fs::create_dir_all(&thumb_dir);
    }

    if !thumb_path.exists() {
        media::generate_image_thumbnail(image_path, &thumb_path)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_ensure_thumbnail_directory_creation() {
        let dir = tempdir().unwrap();
        let img_path = dir.path().join("test.jpg");
        // Create dummy file to avoid "file not found" errors before thumbnailing
        fs::write(&img_path, "").unwrap();

        // We ignore the actual result of thumbnail generation because it requires a valid image,
        // but we verify the directory creation logic.
        let _ = ensure_thumbnail(&img_path);

        assert!(dir.path().join(".thumbnails").exists());
    }

    #[test]
    fn test_ensure_thumbnail_fails_on_missing_file() {
        let dir = tempdir().unwrap();
        let img_path = dir.path().join("missing.jpg");

        let result = ensure_thumbnail(&img_path);
        assert!(result.is_err());
    }
}
