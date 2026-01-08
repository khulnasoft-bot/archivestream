use chrono::Utc;
use uuid::Uuid;

pub struct WarcRecord {
    pub url: String,
    pub timestamp: chrono::DateTime<Utc>,
    pub content: Vec<u8>,
    pub content_type: String,
    pub payload_digest: String,
}

impl WarcRecord {
    pub fn to_warc_string(&self, is_revisit: bool) -> String {
        let record_id = Uuid::new_v4();
        let timestamp = self.timestamp.format("%Y-%m-%dT%H:%M:%SZ").to_string();
        let warc_type = if is_revisit { "revisit" } else { "response" };
        
        format!(
            "WARC/1.0\r\n\
            WARC-Type: {}\r\n\
            WARC-Record-ID: <urn:uuid:{}>\r\n\
            WARC-Date: {}\r\n\
            WARC-Target-URI: {}\r\n\
            WARC-Payload-Digest: sha256:{}\r\n\
            Content-Type: application/http; msgtype=response\r\n\
            Content-Length: {}\r\n\
            \r\n",
            warc_type,
            record_id,
            timestamp,
            self.url,
            self.payload_digest,
            self.content.len()
        )
    }
}
