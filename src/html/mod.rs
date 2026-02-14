mod builder;
mod utils;

use anyhow::{Context, Result};
use log::info;
use std::fs;
use std::path::Path;

pub use self::builder::build_html;
pub use self::utils::is_image;

pub fn generate_gallery(root_path: &Path) -> Result<()> {
    info!("Generating HTML gallery in {:?}", root_path);
    visit_dir(root_path, root_path)?;
    Ok(())
}

fn visit_dir(dir: &Path, root_path: &Path) -> Result<()> {
    if !dir.is_dir() {
        return Ok(());
    }

    let entries = fs::read_dir(dir)?;
    let mut subdirs = Vec::new();
    let mut images = Vec::new();

    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            subdirs.push(path);
        } else if is_image(&path) {
            images.push(path);
        }
    }

    subdirs.sort();
    images.sort();

    // Recurse first
    for subdir in &subdirs {
        visit_dir(subdir, root_path)?;
    }

    // Check if we should generate a flattened view (Depth 2 = Month level)
    let relative_path = dir.strip_prefix(root_path).unwrap_or(Path::new(""));
    let depth = relative_path.components().count();
    let mut flattened_images = Vec::new();

    // Heuristic: If we are at Month level (depth 2) and have subdirectories (Days),
    // collect all images from those subdirectories to show a flattened view.
    if depth == 2 && !subdirs.is_empty() {
        for subdir in &subdirs {
            if let Ok(entries) = fs::read_dir(subdir) {
                let mut dir_images = Vec::new();
                for entry in entries.flatten() {
                    let path = entry.path();
                    if !path.is_dir() && is_image(&path) {
                        // Get path relative to current 'dir' (Month dir)
                        if let Ok(rel) = path.strip_prefix(dir) {
                            dir_images.push(rel.to_path_buf());
                        }
                    }
                }
                dir_images.sort();
                flattened_images.extend(dir_images);
            }
        }
    }

    // Generate HTML for current dir
    // Only generate if there are contents or it's the root
    if !subdirs.is_empty() || !images.is_empty() || dir == root_path {
        let content = build_html(dir, root_path, &subdirs, &images, &flattened_images)?;

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
