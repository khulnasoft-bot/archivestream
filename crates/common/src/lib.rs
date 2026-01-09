use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod extractor;
pub mod zerocopy;
pub mod tracing;

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct Snapshot {
    pub id: Uuid,
    pub url: String,
    pub timestamp: DateTime<Utc>,
    pub warc_file: String,
    pub offset: i64,
    pub length: i64,
    pub sha256: String,
    pub status_code: i16,
    pub content_type: String,
    pub payload_hash: Option<String>,
}

pub mod warc;

pub mod replay {
    use chrono::{DateTime, NaiveDateTime, Utc};
    use serde::{Deserialize, Serialize};
    use url::Url;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ReplayUrl {
        pub timestamp: DateTime<Utc>,
        pub original_url: String,
    }

    impl ReplayUrl {
        pub fn parse(timestamp_str: &str, url_str: &str) -> anyhow::Result<Self> {
            let timestamp = NaiveDateTime::parse_from_str(timestamp_str, "%Y%m%d%H%M%S")?.and_utc();

            // Validate URL
            let _ = Url::parse(url_str)?;

            Ok(Self {
                timestamp,
                original_url: url_str.to_string(),
            })
        }

        pub fn format(&self) -> String {
            format!(
                "/web/{}/{}",
                self.timestamp.format("%Y%m%d%H%M%S"),
                self.original_url
            )
        }
    }
}
