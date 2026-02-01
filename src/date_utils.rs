use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};

pub fn naive_to_utc(y: i32, m: u32, d: u32) -> Option<DateTime<Utc>> {
    let naive = NaiveDateTime::new(
        NaiveDate::from_ymd_opt(y, m, d)?,
        NaiveTime::from_hms_opt(12, 0, 0)?,
    );
    Some(DateTime::from_naive_utc_and_offset(naive, Utc))
}

pub fn timestamp_string_to_date(ts_str: &str) -> Option<DateTime<Utc>> {
    let secs = ts_str.parse::<i64>().ok()?;
    DateTime::from_timestamp(secs, 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Datelike;

    #[test]
    fn test_naive_to_utc_valid() {
        let date = naive_to_utc(2023, 10, 25).unwrap();
        assert_eq!(date.year(), 2023);
        assert_eq!(date.month(), 10);
        assert_eq!(date.day(), 25);
    }

    #[test]
    fn test_naive_to_utc_invalid() {
        assert!(naive_to_utc(2023, 02, 30).is_none()); // Feb 30th does not exist
        assert!(naive_to_utc(2023, 13, 01).is_none()); // Month 13 does not exist
    }

    #[test]
    fn test_timestamp_string_to_date() {
        // 1672531200 = 2023-01-01 00:00:00 UTC
        let date = timestamp_string_to_date("1672531200").unwrap();
        assert_eq!(date.year(), 2023);
        assert_eq!(date.month(), 1);
        assert_eq!(date.day(), 1);
    }

    #[test]
    fn test_timestamp_invalid() {
        assert!(timestamp_string_to_date("invalid").is_none());
    }
}
