use sqlx::PgPool;
use anyhow::Result;

pub struct DedupService {
    pool: PgPool,
}

impl DedupService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn is_duplicate(&self, hash: &str) -> Result<bool> {
        let exists = sqlx::query(
            "SELECT 1 AS x FROM payloads WHERE hash = $1"
        )
        .bind(hash)
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(exists.is_some())
    }

    pub async fn insert_payload(&self, hash: &str, warc_path: &str, offset: u64, size: u64) -> Result<()> {
        sqlx::query(
            "INSERT INTO payloads (hash, warc_path, warc_offset, size) VALUES ($1, $2, $3, $4) ON CONFLICT DO NOTHING"
        )
        .bind(hash)
        .bind(warc_path)
        .bind(offset as i64)
        .bind(size as i64)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
}
