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

pub fn is_video(path: &Path) -> bool {
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_lowercase())
        .unwrap_or_default();

    matches!(ext.as_str(), "mp4" | "mov" | "avi" | "mkv" | "webm" | "m4v")
}

pub fn get_date_from_path(image_path: &Path, root_path: &Path) -> Option<String> {
    let relative = image_path.strip_prefix(root_path).ok()?;
    let components: Vec<_> = relative.components().collect();

    // Expect structure: Year/MonthName/Day/Image.jpg
    if components.len() >= 4 {
        let year_str = components[0].as_os_str().to_string_lossy();
        let month_name = components[1].as_os_str().to_string_lossy();
        let day_str = components[2].as_os_str().to_string_lossy();

        let month_num = match month_name.as_ref() {
            "January" => "01",
            "February" => "02",
            "March" => "03",
            "April" => "04",
            "May" => "05",
            "June" => "06",
            "July" => "07",
            "August" => "08",
            "September" => "09",
            "October" => "10",
            "November" => "11",
            "December" => "12",
            _ => return None,
        };

        // Simple validation: check if year and day look like numbers
        if year_str.chars().all(char::is_numeric) && day_str.chars().all(char::is_numeric) {
            let day_val = day_str.parse::<u32>().unwrap_or(0);
            return Some(format!("{}-{}-{:02}", year_str, month_num, day_val));
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
        let path = root.join("2023/May/20/img.jpg");
        assert_eq!(
            get_date_from_path(&path, root),
            Some("2023-05-20".to_string())
        );

        // Valid path with deep nesting
        let path = root.join("2023/January/01/extra/img.jpg");
        assert_eq!(
            get_date_from_path(&path, root),
            Some("2023-01-01".to_string())
        );

        // Too short
        let path = root.join("2023/May/img.jpg");
        assert_eq!(get_date_from_path(&path, root), None);

        // Numeric month (no longer supported)
        let path = root.join("2023/05/20/img.jpg");
        assert_eq!(get_date_from_path(&path, root), None);
    }
}
