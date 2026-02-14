use std::path::Path;

pub fn is_image(path: &Path) -> bool {
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_lowercase())
        .unwrap_or_default();

    matches!(
        ext.as_str(),
        "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp" | "heic" | "heif" | "tiff"
    )
}

pub fn get_date_from_path(image_path: &Path, root_path: &Path) -> Option<String> {
    let relative = image_path.strip_prefix(root_path).ok()?;
    let components: Vec<_> = relative.components().collect();

    // Expect structure: Year/Month/Day/Image.jpg
    // So we need at least 3 parent directories.
    if components.len() >= 4 {
        let year_str = components[0].as_os_str().to_string_lossy();
        let month_str = components[1].as_os_str().to_string_lossy();
        let day_str = components[2].as_os_str().to_string_lossy();

        // Simple validation: check if they look like numbers
        if year_str.chars().all(char::is_numeric)
            && month_str.chars().all(char::is_numeric)
            && day_str.chars().all(char::is_numeric)
        {
            return Some(format!("{}-{}-{}", year_str, month_str, day_str));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_image() {
        assert!(is_image(Path::new("photo.jpg")));
        assert!(is_image(Path::new("photo.JPG")));
        assert!(is_image(Path::new("image.png")));
        assert!(is_image(Path::new("image.heic")));

        assert!(!is_image(Path::new("video.mp4")));
        assert!(!is_image(Path::new("text.txt")));
        assert!(!is_image(Path::new("no_ext")));
    }

    #[test]
    fn test_get_date_from_path() {
        let root = Path::new("/tmp/output");

        // Valid path
        let path = root.join("2023/05/20/img.jpg");
        assert_eq!(
            get_date_from_path(&path, root),
            Some("2023-05-20".to_string())
        );

        // Valid path with deep nesting (still works if structure is preserved at start)
        let path = root.join("2023/05/20/extra/img.jpg");
        assert_eq!(
            get_date_from_path(&path, root),
            Some("2023-05-20".to_string())
        );

        // Too short
        let path = root.join("2023/05/img.jpg");
        assert_eq!(get_date_from_path(&path, root), None);

        // Non-numeric
        let path = root.join("unknown/folder/img.jpg");
        assert_eq!(get_date_from_path(&path, root), None);

        // Mixed numeric/non-numeric
        let path = root.join("2023/May/20/img.jpg");
        assert_eq!(get_date_from_path(&path, root), None);

        // Path not relative to root
        let path = Path::new("/other/path/2023/05/20/img.jpg");
        assert_eq!(get_date_from_path(&path, root), None);
    }
}
