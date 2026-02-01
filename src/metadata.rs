use anyhow::Result;
use chrono::{DateTime, NaiveDateTime, Utc};
use log::debug;
use regex::Regex;
use std::fs;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use crate::date_utils::{naive_to_utc, timestamp_string_to_date};
use crate::model::PhotoMetadata;

pub struct DateExtractor {
    regex_std: Regex,
    regex_dmy: Regex,
}

impl DateExtractor {
    pub fn new() -> Result<Self> {
        Ok(Self {
            regex_std: Regex::new(r"(\d{4})[-_]?(\d{2})[-_]?(\d{2})")?,
            regex_dmy: Regex::new(r"(\d{2})(\d{2})(\d{4})")?,
        })
    }

    pub fn determine_date(&self, input_path: &Path) -> Option<DateTime<Utc>> {
        let mut path_with_extra_json_ext = PathBuf::from(input_path);
        if let Some(filename) = input_path.file_name() {
            let mut new_name = filename.to_os_string();
            new_name.push(".json");
            path_with_extra_json_ext.set_file_name(new_name);
        }
        let path_with_json_ext = input_path.with_extension("json");

        // Extract Date from metadata json
        if let Some(date) = self
            .parse_json_date(&path_with_extra_json_ext)
            .or_else(|| self.parse_json_date(&path_with_json_ext))
        {
            return Some(date);
        }

        // Extract Date from EXIF
        if let Some(date) = self.get_exif_date(input_path) {
            debug!("Date found in EXIF for: {:?}", input_path.file_name());
            return Some(date);
        }

        // Extract Date from filename
        if let Some(date) = self.parse_filename_date(input_path) {
            return Some(date);
        }

        None
    }

    fn parse_json_date(&self, json_path: &Path) -> Option<DateTime<Utc>> {
        if !json_path.exists() {
            return None;
        }

        debug!("Found JSON metadata: {:?}", json_path);

        let file = fs::File::open(json_path).ok()?;
        let reader = BufReader::new(file);
        let metadata: PhotoMetadata = serde_json::from_reader(reader).ok()?;

        if let Some(taken) = metadata.photo_taken_time {
            return timestamp_string_to_date(&taken.timestamp);
        }

        if let Some(created) = metadata.creation_time {
            return timestamp_string_to_date(&created.timestamp);
        }

        None
    }

    fn get_exif_date(&self, input_path: &Path) -> Option<DateTime<Utc>> {
        let file = fs::File::open(input_path).ok()?;
        let mut bufreader = BufReader::new(&file);
        let exif_reader = exif::Reader::new();

        let exif = exif_reader.read_from_container(&mut bufreader).ok()?;

        let date_tag = exif
            .get_field(exif::Tag::DateTimeOriginal, exif::In::PRIMARY)
            .or_else(|| exif.get_field(exif::Tag::DateTimeDigitized, exif::In::PRIMARY))
            .or_else(|| exif.get_field(exif::Tag::DateTime, exif::In::PRIMARY));

        if let Some(field) = date_tag {
            let date_value = field.display_value().with_unit(&exif).to_string();
            let clean_date_value = date_value.trim();

            if let Ok(naive) = NaiveDateTime::parse_from_str(clean_date_value, "%Y:%m:%d %H:%M:%S")
            {
                return Some(DateTime::from_naive_utc_and_offset(naive, Utc));
            }

            if let Ok(naive) = NaiveDateTime::parse_from_str(clean_date_value, "%Y-%m-%d %H:%M:%S")
            {
                return Some(DateTime::from_naive_utc_and_offset(naive, Utc));
            }
        }

        None
    }

    fn parse_filename_date(&self, input_path: &Path) -> Option<DateTime<Utc>> {
        let filename_str = input_path.file_name()?.to_str()?;

        if let Some(caps) = self.regex_std.captures(filename_str) {
            let y = caps.get(1)?.as_str().parse::<i32>().ok()?;
            if y > 1990 && y < 2030 {
                let m = caps.get(2)?.as_str().parse::<u32>().ok()?;
                let d = caps.get(3)?.as_str().parse::<u32>().ok()?;
                if let Some(dt) = naive_to_utc(y, m, d) {
                    return Some(dt);
                }
            }
        }

        if let Some(caps) = self.regex_dmy.captures(filename_str) {
            let d = caps.get(1)?.as_str().parse::<u32>().ok()?;
            let m = caps.get(2)?.as_str().parse::<u32>().ok()?;
            let y = caps.get(3)?.as_str().parse::<i32>().ok()?;

            if y > 1990
                && y < 2030
                && let Some(dt) = naive_to_utc(y, m, d)
            {
                return Some(dt);
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Datelike;

    #[test]
    fn test_filename_date_extraction_std() {
        let extractor = DateExtractor::new().unwrap();

        let path = Path::new("IMG_20230520_120000.jpg");
        let date = extractor.parse_filename_date(path).unwrap();
        assert_eq!(date.year(), 2023);
        assert_eq!(date.month(), 5);
        assert_eq!(date.day(), 20);

        let path = Path::new("2022-12-01.jpg");
        let date = extractor.parse_filename_date(path).unwrap();
        assert_eq!(date.year(), 2022);
        assert_eq!(date.month(), 12);
        assert_eq!(date.day(), 1);
    }

    #[test]
    fn test_filename_date_extraction_dmy() {
        let extractor = DateExtractor::new().unwrap();

        // WhatsApp style sometimes uses this or similar
        let path = Path::new("IMG-25102023-WA0001.jpg");
        let date = extractor.parse_filename_date(path).unwrap();
        assert_eq!(date.year(), 2023);
        assert_eq!(date.month(), 10);
        assert_eq!(date.day(), 25);
    }

    #[test]
    fn test_filename_no_date() {
        let extractor = DateExtractor::new().unwrap();
        let path = Path::new("random_image.jpg");
        assert!(extractor.parse_filename_date(path).is_none());
    }
}
