use reqwest::Client;
use reqwest::header::RANGE;
use anyhow::{Result, anyhow};
use bytes::Bytes;

pub struct WarcReader {
    client: Client,
    base_url: String, // MinIO/S3 entry point
}

impl WarcReader {
    pub fn new(base_url: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
        }
    }

    pub async fn read_record(&self, filename: &str, offset: u64, length: u64) -> Result<Bytes> {
        let url = format!("{}/{}", self.base_url, filename);
        
        let response = self.client
            .get(&url)
            .header(RANGE, format!("bytes={}-{}", offset, offset + length - 1))
            .send()
            .await?;

        if !response.status().is_success() && response.status() != 206 {
            return Err(anyhow!("Failed to fetch WARC record: status {}", response.status()));
        }

        let body = response.bytes().await?;
        
        // Basic parsing: Skip WARC headers to get the HTTP response
        // In a real implementation, we'd use a WARC parser crate to handle headers properly
        // For now, we skip until the first \r\n\r\n (WARC headers end)
        // and then the next \r\n\r\n (HTTP headers end) if we want just the body,
        // but typically Replay serves the whole HTTP response (Headers + Body).
        
        // Find \r\n\r\n which separates WARC headers from HTTP payload
        let warc_header_end = body.windows(4)
            .position(|w| w == b"\r\n\r\n")
            .ok_or_else(|| anyhow!("Invalid WARC record: missing header separator"))?;
        
        let http_payload = body.slice((warc_header_end + 4)..);
        
        Ok(http_payload)
    }
}
