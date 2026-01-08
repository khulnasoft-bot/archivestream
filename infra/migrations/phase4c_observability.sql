-- ArchiveStream Observability Schema (Phase 4C)

-- 1. Update url_frontier to support depth tracking
ALTER TABLE url_frontier ADD COLUMN IF NOT EXISTS depth INT DEFAULT 0;
ALTER TABLE url_frontier ADD COLUMN IF NOT EXISTS last_status INT;

-- 2. Create crawl_events for telemetry
CREATE TABLE IF NOT EXISTS crawl_events (
    id BIGSERIAL PRIMARY KEY,
    domain TEXT NOT NULL,
    url TEXT NOT NULL,
    status TEXT NOT NULL, -- 'success', 'error', 'timeout'
    http_status INT,
    duration_ms INT,
    timestamp TIMESTAMPTZ DEFAULT NOW()
);

-- 3. Optimize for dashboard queries
CREATE INDEX IF NOT EXISTS idx_frontier_domain_depth ON url_frontier(domain, depth);
CREATE INDEX IF NOT EXISTS idx_events_timestamp ON crawl_events(timestamp);
CREATE INDEX IF NOT EXISTS idx_events_domain ON crawl_events(domain);
