-- Initial Schema for ArchiveStream Crawler

-- URL Frontier
CREATE TABLE IF NOT EXISTS url_frontier (
    url TEXT PRIMARY KEY,
    domain TEXT NOT NULL,
    priority INT DEFAULT 0,
    next_fetch_at TIMESTAMPTZ DEFAULT NOW(),
    fetch_attempts INT DEFAULT 0,
    leased_until TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Indexes for Frontier claiming performance
CREATE INDEX IF NOT EXISTS idx_frontier_claim 
ON url_frontier(priority DESC, created_at ASC) 
WHERE leased_until IS NULL AND next_fetch_at <= NOW();

-- Snapshots (Archived Content)
CREATE TABLE IF NOT EXISTS snapshots (
    id BIGSERIAL PRIMARY KEY,
    url TEXT NOT NULL,
    timestamp TIMESTAMPTZ DEFAULT NOW(),
    sha256 TEXT NOT NULL,
    http_status INT,
    content_type TEXT,
    data BYTEA
);

CREATE INDEX IF NOT EXISTS idx_snapshots_url ON snapshots(url);
CREATE INDEX IF NOT EXISTS idx_snapshots_sha256 ON snapshots(sha256);
