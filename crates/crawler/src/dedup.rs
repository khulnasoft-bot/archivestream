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
        let exists = sqlx::query!(
            "SELECT 1 as x FROM payloads WHERE hash = $1",
            hash
        )
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(exists.is_some())
    }

    pub async fn insert_payload(&self, hash: &str, warc_path: &str, offset: u64, size: u64) -> Result<()> {
        sqlx::query!(
            "INSERT INTO payloads (hash, warc_path, warc_offset, size) VALUES ($1, $2, $3, $4) ON CONFLICT DO NOTHING",
            hash,
            warc_path,
            offset as i64,
            size as i64
        )
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
}
