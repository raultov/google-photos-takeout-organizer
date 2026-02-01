use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};

pub fn naive_to_utc(y: i32, m: u32, d: u32) -> Option<DateTime<Utc>> {
    let naive = NaiveDateTime::new(
        NaiveDate::from_ymd_opt(y, m, d)?,
        NaiveTime::from_hms_opt(12, 0, 0)?
    );
    Some(DateTime::from_naive_utc_and_offset(naive, Utc))
}

pub fn timestamp_string_to_date(ts_str: &str) -> Option<DateTime<Utc>> {
    let secs = ts_str.parse::<i64>().ok()?;
    DateTime::from_timestamp(secs, 0)
}
