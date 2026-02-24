use super::builder;
use super::utils::{is_image, is_video};
use anyhow::{Context, Result};
use indicatif::ProgressBar;
use std::fs;
use std::path::Path;

/// Recursively visits directories to generate gallery HTML files.
pub fn visit_dir(dir: &Path, root_path: &Path, pb: &ProgressBar) -> Result<()> {
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
        let content =
            builder::build_html(dir, root_path, &subdirs, &media_files, &flattened_media)?;

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_visit_dir_creates_collection_html() {
        let root = tempdir().unwrap();
        let pb = ProgressBar::hidden();

        // Crear una imagen para que se genere el HTML
        fs::write(root.path().join("image.jpg"), "").unwrap();

        visit_dir(root.path(), root.path(), &pb).unwrap();

        assert!(root.path().join("collection.html").exists());
    }

    #[test]
    fn test_month_sorting_logic() {
        let root = tempdir().unwrap();
        let year_dir = root.path().join("2023");
        fs::create_dir_all(&year_dir).unwrap();

        // Crear meses fuera de orden alfabético
        fs::create_dir(year_dir.join("December")).unwrap();
        fs::create_dir(year_dir.join("January")).unwrap();
        fs::create_dir(year_dir.join("March")).unwrap();

        // Añadir una imagen en la raíz para disparar la generación
        fs::write(root.path().join("img.jpg"), "").unwrap();

        let pb = ProgressBar::hidden();
        visit_dir(root.path(), root.path(), &pb).unwrap();

        // Verificamos que se generó el index del año
        let year_html = fs::read_to_string(year_dir.join("index.html")).unwrap();

        // En el HTML, January debería aparecer antes que December a pesar del orden alfabético
        let jan_pos = year_html.find("January").unwrap();
        let dec_pos = year_html.find("December").unwrap();
        assert!(jan_pos < dec_pos);
    }

    #[test]
    fn test_flattened_view_at_month_level() {
        let root = tempdir().unwrap();
        // Estructura: Root / 2023 / January / 01 / img.jpg
        let day_dir = root.path().join("2023/January/01");
        fs::create_dir_all(&day_dir).unwrap();
        fs::write(day_dir.join("photo.jpg"), "").unwrap();

        let pb = ProgressBar::hidden();
        visit_dir(root.path(), root.path(), &pb).unwrap();

        // El index del mes (January) debería contener la foto del día (01)
        let month_html = fs::read_to_string(root.path().join("2023/January/index.html")).unwrap();
        assert!(month_html.contains("01/photo.jpg"));
    }
}
