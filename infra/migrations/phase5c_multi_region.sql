-- Phase 5C: Multi-Region Frontier Schema

-- 1. Add region affinity and worker tracking to url_frontier
ALTER TABLE url_frontier ADD COLUMN IF NOT EXISTS preferred_region TEXT DEFAULT 'us-east-1';
ALTER TABLE url_frontier ADD COLUMN IF NOT EXISTS leased_by_worker TEXT;
ALTER TABLE url_frontier ADD COLUMN IF NOT EXISTS leased_region TEXT;

-- 2. Create rate limiting table
CREATE TABLE IF NOT EXISTS rate_limits (
    domain TEXT NOT NULL,
    region TEXT NOT NULL,
    window_start TIMESTAMPTZ NOT NULL,
    request_count INT DEFAULT 0,
    PRIMARY KEY (domain, region, window_start)
);

-- 3. Create worker registry for multi-region observability
CREATE TABLE IF NOT EXISTS workers (
    worker_id TEXT PRIMARY KEY,
    region TEXT NOT NULL,
    last_heartbeat TIMESTAMPTZ DEFAULT NOW(),
    active_tasks INT DEFAULT 0,
    total_processed BIGINT DEFAULT 0
);

-- 4. Indexes for efficient regional queries
CREATE INDEX IF NOT EXISTS idx_frontier_region ON url_frontier(preferred_region, next_fetch_at) WHERE leased_until IS NULL;
CREATE INDEX IF NOT EXISTS idx_rate_limits_lookup ON rate_limits(domain, region, window_start);
CREATE INDEX IF NOT EXISTS idx_workers_region ON workers(region, last_heartbeat);

-- 5. Create view for regional frontier stats
CREATE OR REPLACE VIEW regional_frontier_stats AS
SELECT 
    preferred_region as region,
    COUNT(*) as pending_urls,
    COUNT(DISTINCT domain) as unique_domains,
    MIN(depth) as min_depth,
    MAX(depth) as max_depth
FROM url_frontier
WHERE leased_until IS NULL OR leased_until < NOW()
GROUP BY preferred_region;
