use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct GoogleTimestamp {
    pub timestamp: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PhotoMetadata {
    pub photo_taken_time: Option<GoogleTimestamp>,
    pub creation_time: Option<GoogleTimestamp>,
}
