use anyhow::Result;
use std::path::{Path, PathBuf};

use super::utils::{get_date_from_path, is_video};

pub fn build_html(
    current_dir: &Path,
    root_path: &Path,
    subdirs: &[PathBuf],
    images: &[PathBuf],
    flattened_images: &[PathBuf],
) -> Result<String> {
    let relative_path = current_dir.strip_prefix(root_path).unwrap_or(Path::new(""));
    let title = if relative_path.as_os_str().is_empty() {
        "Photo Collection".to_string()
    } else {
        relative_path.display().to_string()
    };

    let has_flattened = !flattened_images.is_empty();

    let mut html = String::new();

    // Header
    let header_tmpl = include_str!("templates/header.html");
    let css = include_str!("templates/styles.css");
    html.push_str(
        &header_tmpl
            .replace("{title}", &title)
            .replace("{styles}", css),
    );

    // Toggle Button
    if has_flattened {
        html.push_str(include_str!("templates/toggle_btn.html"));
        html.push('\n');
    }

    // Breadcrumb
    html.push_str("    <div class=\"breadcrumb\">\n");

    // Link to root
    let root_link = if current_dir == root_path {
        "collection.html".to_string()
    } else {
        let depth = relative_path.components().count();
        let mut ups = String::new();
        for _ in 0..depth {
            ups.push_str("../");
        }
        ups + "collection.html"
    };

    html.push_str(&format!(r#"        <a href="{}">Home</a>"#, root_link));

    if current_dir != root_path {
        html.push_str(" <span>/</span> ");
        let components: Vec<_> = relative_path.components().collect();
        for (i, component) in components.iter().enumerate() {
            let name = component.as_os_str().to_string_lossy();

            if i == components.len() - 1 {
                html.push_str(&format!("<span>{}</span>", name));
            } else {
                // Calculate relative path from current_dir back to this component
                let levels_up = components.len() - 1 - i;
                let mut path_str = String::new();
                for _ in 0..levels_up {
                    path_str.push_str("../");
                }
                path_str.push_str("index.html");
                html.push_str(&format!(
                    r#"<a href="{}">{}</a> <span>/</span> "#,
                    path_str, name
                ));
            }
        }
    }

    html.push_str("    </div>\n");

    html.push_str(&format!("    <h1>{}</h1>\n", title));

    // Flattened Gallery Container
    if has_flattened {
        html.push_str(
            r#"    <div id="flattened-gallery" class="gallery">
"#,
        );
        html.push_str(&generate_images_html(images, current_dir, root_path));

        for image in flattened_images {
            let path_str = image.to_string_lossy();
            let filename = image.file_name().unwrap().to_string_lossy();

            let full_path = current_dir.join(image);
            let date_str = get_date_from_path(&full_path, root_path).unwrap_or_default();
            let is_vid = is_video(&full_path);

            let display_src = if is_vid {
                let parent = image.parent().unwrap_or(Path::new(""));
                let thumb_path = parent.join(".thumbnails").join(format!("{}.jpg", filename));
                thumb_path.to_string_lossy().to_string()
            } else {
                path_str.to_string()
            };

            html.push_str(&generate_photo_html(
                &path_str,
                &display_src,
                &filename,
                &date_str,
                is_vid,
            ));
        }
        html.push_str("    </div>\n");
    }

    // Directory View Container
    let dir_view_style = if has_flattened { "display: none;" } else { "" };
    html.push_str(&format!(
        r#"    <div id="directory-view" style="{}">
"#,
        dir_view_style
    ));

    // Directories
    if !subdirs.is_empty() {
        html.push_str("    <h2>Directories</h2>\n");
        html.push_str(
            r#"    <div class="directories">
"#,
        );
        let dir_tmpl = include_str!("templates/directory_card.html");
        for subdir in subdirs {
            let dirname = subdir.file_name().unwrap().to_string_lossy();
            let href = format!("{}/index.html", dirname);
            html.push_str(
                &dir_tmpl
                    .replace("{href}", &href)
                    .replace("{name}", &dirname),
            );
        }
        html.push_str("    </div>\n");
    }

    // Photos (Direct)
    if !images.is_empty() {
        html.push_str("    <h2>Photos</h2>\n");
        html.push_str(
            r#"    <div class="gallery">
"#,
        );
        html.push_str(&generate_images_html(images, current_dir, root_path));
        html.push_str("    </div>\n");
    }

    html.push_str("    </div>\n"); // End directory-view

    // Inject Modal HTML
    html.push_str(include_str!("templates/modal.html"));

    // Inject JS
    html.push_str("    <script>\n");
    html.push_str(include_str!("templates/script.js"));
    html.push_str("    </script>\n");

    html.push_str("</body>\n</html>");

    Ok(html)
}

fn generate_photo_html(
    src: &str,
    display_src: &str,
    alt: &str,
    date: &str,
    is_video: bool,
) -> String {
    let tmpl = include_str!("templates/photo_card.html");
    let type_str = if is_video { "video" } else { "image" };
    let play_icon = if is_video {
        "<div class=\"play-icon\">▶</div>"
    } else {
        ""
    };

    tmpl.replace("{src}", src)
        .replace("{display_src}", display_src)
        .replace("{alt}", alt)
        .replace("{date}", date)
        .replace("{type}", type_str)
        .replace("{play_icon}", play_icon)
}

fn generate_images_html(images: &[PathBuf], current_dir: &Path, root_path: &Path) -> String {
    let mut html = String::new();
    for image in images {
        let filename = image.file_name().unwrap().to_string_lossy();
        let full_path = current_dir.join(image);
        let date_str = get_date_from_path(&full_path, root_path).unwrap_or_default();
        let is_vid = is_video(&full_path);

        let display_src = if is_vid {
            let thumb_path = Path::new(".thumbnails").join(format!("{}.jpg", filename));
            thumb_path.to_string_lossy().to_string()
        } else {
            filename.to_string()
        };

        html.push_str(&generate_photo_html(
            &filename,
            &display_src,
            &filename,
            &date_str,
            is_vid,
        ));
    }
    html
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_photo_html() {
        let html = generate_photo_html("img.jpg", "img.jpg", "img.jpg", "2023-01-01", false);
        assert!(html.contains("href=\"img.jpg\""));
        assert!(html.contains("src=\"img.jpg\""));
        assert!(html.contains("data-type=\"image\""));
        assert!(!html.contains("play-icon"));

        let html_vid = generate_photo_html(
            "vid.mp4",
            ".thumbnails/vid.mp4.jpg",
            "vid.mp4",
            "2023-01-01",
            true,
        );
        assert!(html_vid.contains("href=\"vid.mp4\""));
        assert!(html_vid.contains("src=\".thumbnails/vid.mp4.jpg\""));
        assert!(html_vid.contains("data-type=\"video\""));
        assert!(html_vid.contains("play-icon"));
    }

    #[test]
    fn test_build_html_basic() {
        let root = Path::new("/tmp/root");
        let subdirs = vec![];
        let images = vec![];
        let flattened = vec![];

        let html = build_html(root, root, &subdirs, &images, &flattened).unwrap();

        assert!(html.contains("<h1>Photo Collection</h1>"));
        assert!(!html.contains("id=\"toggle-btn\"")); // No toggle button
        assert!(html.contains("id=\"directory-view\" style=\"\"")); // Visible dir view
    }

    #[test]
    fn test_build_html_with_flattened() {
        let root = Path::new("/tmp/root");
        let current = root.join("2023/01");
        let subdirs = vec![current.join("01")]; // One subdir
        let images = vec![];
        let flattened = vec![PathBuf::from("01/img.jpg")];

        let html = build_html(&current, root, &subdirs, &images, &flattened).unwrap();

        assert!(html.contains("<title>2023/01</title>"));
        assert!(html.contains("id=\"toggle-btn\"")); // Toggle present
        assert!(html.contains("id=\"directory-view\" style=\"display: none;\"")); // Hidden dir view
        assert!(html.contains("id=\"flattened-gallery\""));
        assert!(html.contains("01/img.jpg"));
    }

    #[test]
    fn test_build_html_breadcrumbs() {
        let root = Path::new("/tmp/root");
        let current = root.join("2023/01/01");
        let html = build_html(&current, root, &[], &[], &[]).unwrap();

        assert!(html.contains("href=\"../../../collection.html\"")); // 3 levels up
        assert!(html.contains("2023"));
        assert!(html.contains("01"));
    }
}
