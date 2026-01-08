use sqlx::PgPool;
use anyhow::Result;
use chrono::{DateTime, Utc};
use url::Url;

pub struct FrontierService {
    pool: PgPool,
}

#[derive(Debug, sqlx::FromRow)]
pub struct FrontierUrl {
    pub url: String,
    pub domain: Option<String>,
    pub depth: i32,
}

impl FrontierService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn add_url(&self, url: &str, priority: i32, depth: i32) -> Result<()> {
        let domain = Url::parse(url).ok().and_then(|u| u.domain().map(|d| d.to_string()));
        
        sqlx::query!(
            "INSERT INTO url_frontier (url, domain, priority, depth) VALUES ($1, $2, $3, $4) ON CONFLICT (url) DO NOTHING",
            url,
            domain,
            priority,
            depth
        )
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }

    pub async fn claim_urls(&self, limit: i32) -> Result<Vec<FrontierUrl>> {
        let urls = sqlx::query_as!(
            FrontierUrl,
            r#"
            UPDATE url_frontier
            SET leased_until = now() + interval '1 minute'
            WHERE url IN (
                SELECT url FROM url_frontier
                WHERE (leased_until IS NULL OR leased_until < now())
                  AND next_fetch_at <= now()
                ORDER BY priority DESC, created_at ASC
                LIMIT $1
                FOR UPDATE SKIP LOCKED
            )
            RETURNING url, domain, depth as "depth!"
            "#,
            limit as i64
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(urls)
    }

    pub async fn complete(&self, url: &str) -> Result<()> {
        sqlx::query!("DELETE FROM url_frontier WHERE url = $1", url)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn fail(&self, url: &str, backoff_seconds: i64) -> Result<()> {
        let next_fetch = Utc::now() + chrono::Duration::seconds(backoff_seconds);
        sqlx::query!(
            "UPDATE url_frontier SET fetch_attempts = fetch_attempts + 1, next_fetch_at = $1, leased_until = NULL WHERE url = $2",
            next_fetch,
            url
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn track_event(&self, url: &str, status: &str, http_status: Option<i32>, duration_ms: i32) -> Result<()> {
        let domain = Url::parse(url).ok().and_then(|u| u.domain().map(|d| d.to_string())).unwrap_or_default();
        sqlx::query!(
            "INSERT INTO crawl_events (domain, url, status, http_status, duration_ms) VALUES ($1, $2, $3, $4, $5)",
            domain,
            url,
            status,
            http_status,
            duration_ms
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
