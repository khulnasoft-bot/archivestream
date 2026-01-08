use sqlx::PgPool;
use chrono::{Utc, Duration};
use anyhow::Result;

/// Token bucket rate limiter with per-domain and per-region limits
pub struct RateLimiter {
    pool: PgPool,
    global_limit_per_domain: i32,
    regional_limit_per_domain: i32,
    window_seconds: i64,
}

impl RateLimiter {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            global_limit_per_domain: 10,      // 10 requests per minute globally
            regional_limit_per_domain: 5,     // 5 requests per minute per region
            window_seconds: 60,
        }
    }

    /// Check if a request is allowed for a domain in a specific region
    pub async fn check_and_increment(&self, domain: &str, region: &str) -> Result<bool> {
        let window_start = self.current_window_start();

        // 1. Check regional limit
        let regional_count = self.get_count(domain, region, window_start).await?;
        if regional_count >= self.regional_limit_per_domain {
            tracing::warn!("Regional rate limit exceeded for {} in {}", domain, region);
            return Ok(false);
        }

        // 2. Check global limit (sum across all regions)
        let global_count = self.get_global_count(domain, window_start).await?;
        if global_count >= self.global_limit_per_domain {
            tracing::warn!("Global rate limit exceeded for {}", domain);
            return Ok(false);
        }

        // 3. Increment counter
        self.increment(domain, region, window_start).await?;
        Ok(true)
    }

    async fn get_count(&self, domain: &str, region: &str, window_start: chrono::DateTime<Utc>) -> Result<i32> {
        let result = sqlx::query!(
            "SELECT request_count FROM rate_limits WHERE domain = $1 AND region = $2 AND window_start = $3",
            domain,
            region,
            window_start
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(result.map(|r| r.request_count).unwrap_or(0))
    }

    async fn get_global_count(&self, domain: &str, window_start: chrono::DateTime<Utc>) -> Result<i32> {
        let result = sqlx::query!(
            "SELECT SUM(request_count) as total FROM rate_limits WHERE domain = $1 AND window_start = $2",
            domain,
            window_start
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(result.total.unwrap_or(0) as i32)
    }

    async fn increment(&self, domain: &str, region: &str, window_start: chrono::DateTime<Utc>) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO rate_limits (domain, region, window_start, request_count)
            VALUES ($1, $2, $3, 1)
            ON CONFLICT (domain, region, window_start)
            DO UPDATE SET request_count = rate_limits.request_count + 1
            "#,
            domain,
            region,
            window_start
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    fn current_window_start(&self) -> chrono::DateTime<Utc> {
        let now = Utc::now();
        let seconds_since_epoch = now.timestamp();
        let window_aligned = (seconds_since_epoch / self.window_seconds) * self.window_seconds;
        chrono::DateTime::from_timestamp(window_aligned, 0).unwrap_or(now)
    }

    /// Cleanup old rate limit windows (run periodically)
    pub async fn cleanup_old_windows(&self) -> Result<()> {
        let cutoff = Utc::now() - Duration::hours(1);
        sqlx::query!(
            "DELETE FROM rate_limits WHERE window_start < $1",
            cutoff
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
