use anyhow::{Context, Result};
use chrono::{DateTime, Datelike, Utc};
use log::{debug, info, warn};
use std::fs;
use std::io;
use std::path::Path;

#[derive(Debug, PartialEq)]
pub enum FileAction {
    New,
    Updated,
    Skipped,
}

pub fn should_process_file(path: &Path) -> bool {
    if path.is_dir() {
        return false;
    }

    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_lowercase())
        .unwrap_or_default();

    if ext == "json" || ext.is_empty() {
        return false;
    }

    true
}

pub fn is_archive(path: &Path) -> bool {
    let filename = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
    filename.ends_with(".zip") || filename.ends_with(".tar.gz") || filename.ends_with(".tgz")
}

pub fn extract_archive(archive_path: &Path, extract_to: &Path) -> Result<()> {
    let filename = archive_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("");

    if filename.ends_with(".zip") {
        info!("Extracting ZIP archive: {:?}", archive_path);
        let file = fs::File::open(archive_path)?;
        let mut archive = zip::ZipArchive::new(file)?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let outpath = match file.enclosed_name() {
                Some(path) => extract_to.join(path),
                None => continue,
            };

            if (*file.name()).ends_with('/') {
                fs::create_dir_all(&outpath)?;
            } else {
                if let Some(p) = outpath.parent()
                    && !p.exists()
                {
                    fs::create_dir_all(p)?;
                }
                let mut outfile = fs::File::create(&outpath)?;
                io::copy(&mut file, &mut outfile)?;
            }
        }
    } else if filename.ends_with(".tar.gz") || filename.ends_with(".tgz") {
        info!("Extracting TAR.GZ archive: {:?}", archive_path);
        let tar_gz = fs::File::open(archive_path)?;
        let tar = flate2::read::GzDecoder::new(tar_gz);
        let mut archive = tar::Archive::new(tar);
        archive.unpack(extract_to)?;
    }

    Ok(())
}

pub fn process_file(
    input_path: &Path,
    output_path: &Path,
    date: Option<DateTime<Utc>>,
    unknown_dir: &str,
) -> Result<FileAction> {
    let dest_folder = match date {
        Some(date) => {
            let month_name = match date.month() {
                1 => "January",
                2 => "February",
                3 => "March",
                4 => "April",
                5 => "May",
                6 => "June",
                7 => "July",
                8 => "August",
                9 => "September",
                10 => "October",
                11 => "November",
                12 => "December",
                _ => "Unknown",
            };
            output_path.join(format!("{}/{}/{:02}", date.year(), month_name, date.day()))
        }
        None => {
            warn!(
                "Date unknown for file: {:?}",
                input_path.file_name().unwrap_or_default()
            );
            output_path.join(unknown_dir)
        }
    };

    fs::create_dir_all(&dest_folder).context("Failed to create destination folder")?;

    if let Some(filename) = input_path.file_name() {
        let dest_path = dest_folder.join(filename);

        return if dest_path.exists() {
            let input_metadata = fs::metadata(input_path)?;
            let dest_metadata = fs::metadata(&dest_path)?;

            if input_metadata.len() != dest_metadata.len() {
                info!("Updating file (size changed): {:?}", filename);
                fs::copy(input_path, &dest_path).with_context(|| {
                    format!("Failed to copy file {:?} to {:?}", input_path, dest_path)
                })?;
                Ok(FileAction::Updated)
            } else {
                debug!(
                    "Skipping file (already exists and same size): {:?}",
                    filename
                );
                Ok(FileAction::Skipped)
            }
        } else {
            fs::copy(input_path, &dest_path).with_context(|| {
                format!("Failed to copy file {:?} to {:?}", input_path, dest_path)
            })?;
            debug!(
                "Copied {:?} -> {:?}",
                input_path.file_name().unwrap(),
                dest_folder
            );
            Ok(FileAction::New)
        };
    }

    Ok(FileAction::Skipped)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_process_file() {
        assert!(should_process_file(Path::new("photo.jpg")));
        assert!(should_process_file(Path::new("video.mp4")));
        assert!(should_process_file(Path::new("IMAGE.PNG")));

        // Should reject
        assert!(!should_process_file(Path::new("metadata.json")));
        assert!(!should_process_file(Path::new("no_extension")));
        assert!(!should_process_file(Path::new(".hidden")));
    }

    #[test]
    fn test_extract_archive_zip() {
        use std::io::Write;
        let temp_dir = tempfile::tempdir().unwrap();
        let archive_path = temp_dir.path().join("test.zip");
        let extract_to = temp_dir.path().join("extracted_zip");

        // Create a dummy zip file
        let file = std::fs::File::create(&archive_path).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);
        zip.start_file("test.txt", options).unwrap();
        zip.write_all(b"hello zip world").unwrap();
        zip.finish().unwrap();

        // Extract it
        extract_archive(&archive_path, &extract_to).unwrap();

        // Verify
        let extracted_file = extract_to.join("test.txt");
        assert!(extracted_file.exists());
        assert_eq!(
            std::fs::read_to_string(extracted_file).unwrap(),
            "hello zip world"
        );
    }

    #[test]
    fn test_extract_archive_tar_gz() {
        let temp_dir = tempfile::tempdir().unwrap();
        let archive_path = temp_dir.path().join("test.tar.gz");
        let extract_to = temp_dir.path().join("extracted_tar_gz");

        // Create a dummy tar.gz file
        let file = std::fs::File::create(&archive_path).unwrap();
        let enc = flate2::write::GzEncoder::new(file, flate2::Compression::default());
        let mut tar_builder = tar::Builder::new(enc);

        let mut header = tar::Header::new_gnu();
        header.set_path("test_tar.txt").unwrap();
        header.set_size(14);
        header.set_cksum();
        tar_builder
            .append(&header, "hello targz ok".as_bytes())
            .unwrap();
        // Fully flush the encoder to disk
        tar_builder.into_inner().unwrap().finish().unwrap();

        // Extract it
        extract_archive(&archive_path, &extract_to).unwrap();

        // Verify
        let extracted_file = extract_to.join("test_tar.txt");
        assert!(extracted_file.exists());
        assert_eq!(
            std::fs::read_to_string(extracted_file).unwrap(),
            "hello targz ok"
        );
    }
}
