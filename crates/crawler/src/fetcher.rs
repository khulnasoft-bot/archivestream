use sha2::{Sha256, Digest};
use futures::StreamExt;
use reqwest::Client;
use crate::warc::WarcRecord;
use bytes::Bytes;

pub struct Fetcher {
    client: Client,
}

impl Fetcher {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .user_agent("ArchiveStream/0.1.0 (+https://github.com/ArchiveStream/ArchiveStream)")
                .build()
                .unwrap(),
        }
    }

    pub async fn fetch(&self, url: &str) -> anyhow::Result<WarcRecord> {
        let response = self.client.get(url).send().await?;
        let status = response.status();
        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("text/html")
            .to_string();
        
        // Streaming body to compute hash without buffering everything if possible
        // Actually, for WARC record we need the full content too.
        let mut hasher = Sha256::new();
        let mut content = Vec::new();
        let mut stream = response.bytes_stream();

        while let Some(item) = stream.next().await {
            let chunk: Bytes = item?;
            hasher.update(&chunk);
            content.extend_from_slice(&chunk);
        }

        let payload_digest = format!("{:x}", hasher.finalize());
        
        Ok(WarcRecord {
            url: url.to_string(),
            timestamp: chrono::Utc::now(),
            content,
            content_type,
            status_code: status.as_u16(),
            payload_digest,
        })
    }
}
