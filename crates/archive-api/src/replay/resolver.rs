use sqlx::PgPool;
use archive_common::Snapshot;
use chrono::{DateTime, Utc};
use anyhow::Result;

pub struct Resolver {
    pool: PgPool,
}

impl Resolver {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn resolve(&self, url: &str, timestamp: DateTime<Utc>) -> Result<Option<Snapshot>> {
        let snapshot = sqlx::query_as!(
            Snapshot,
            r#"
            SELECT 
                s.id, s.url, s.timestamp, 
                COALESCE(p.warc_path, s.warc_file) as "warc_file!",
                COALESCE(p.warc_offset, s.offset) as "offset!",
                s.length as "length!", s.sha256, s.status_code as "status_code!", s.content_type,
                s.payload_hash
            FROM snapshots s
            LEFT JOIN payloads p ON s.payload_hash = p.hash
            WHERE s.url = $1 AND s.timestamp <= $2
            ORDER BY s.timestamp DESC
            LIMIT 1
            "#,
            url,
            timestamp
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(snapshot)
    }
}
