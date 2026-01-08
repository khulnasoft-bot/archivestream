# Multi-Region Architecture

## Overview
ArchiveStream Phase 5C enables **globally distributed crawling** with regional worker pools, shared frontier state, and intelligent domain affinity routing.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                   Global Frontier Layer                      │
│              (CockroachDB / Postgres Citus)                  │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │  us-east-1   │  │   eu-west-1  │  │  ap-south-1  │      │
│  │  Partition   │  │  Partition   │  │  Partition   │      │
│  └──────────────┘  └──────────────┘  └──────────────┘      │
└─────────────────────────────────────────────────────────────┘
                            │
          ┌─────────────────┼─────────────────┐
          │                 │                 │
    ┌─────▼─────┐     ┌─────▼─────┐     ┌─────▼─────┐
    │  Region A │     │  Region B │     │  Region C │
    │  Workers  │     │  Workers  │     │  Workers  │
    │  (Rust)   │     │  (Rust)   │     │  (Rust)   │
    └─────┬─────┘     └─────┬─────┘     └─────┬─────┘
          │                 │                 │
          └─────────────────┼─────────────────┘
                            │
                    ┌───────▼────────┐
                    │  Global S3 /   │
                    │  MinIO Cluster │
                    └────────────────┘
```

## Key Design Principles

### 1. Domain Affinity
- Domains are hashed to preferred regions using consistent hashing
- Example: `nytimes.com` → `us-east-1`, `bbc.co.uk` → `eu-west-1`
- Reduces cross-region latency and respects geo-blocking

### 2. Atomic Leasing
- Workers claim URLs using `FOR UPDATE SKIP LOCKED`
- Lease timeout ensures failed workers don't block queue
- Regional preference: workers prioritize local partition

### 3. Global Rate Limiting
- Per-domain token bucket (e.g., 10 req/min globally)
- Per-region token bucket (e.g., 5 req/min per region)
- Implemented via Redis or Postgres-based counters

### 4. Fault Tolerance
- Region outages don't lose work (replicated frontier)
- Workers in healthy regions pick up orphaned tasks
- WARC storage uses multi-region replication

## Database Schema Extensions

```sql
-- Add region affinity to url_frontier
ALTER TABLE url_frontier ADD COLUMN preferred_region TEXT DEFAULT 'us-east-1';
ALTER TABLE url_frontier ADD COLUMN leased_by_worker TEXT;
ALTER TABLE url_frontier ADD COLUMN leased_region TEXT;

-- Create rate limit tracking table
CREATE TABLE rate_limits (
    domain TEXT NOT NULL,
    region TEXT NOT NULL,
    window_start TIMESTAMPTZ NOT NULL,
    request_count INT DEFAULT 0,
    PRIMARY KEY (domain, region, window_start)
);

-- Create worker registry for observability
CREATE TABLE workers (
    worker_id TEXT PRIMARY KEY,
    region TEXT NOT NULL,
    last_heartbeat TIMESTAMPTZ DEFAULT NOW(),
    active_tasks INT DEFAULT 0
);
```

## Deployment

### Single-Region (Development)
```bash
docker-compose up
```

### Multi-Region (Production)
```bash
# Deploy CockroachDB cluster
cockroach start --advertise-addr=<region-ip> --join=<cluster-nodes>

# Deploy workers per region
REGION=us-east-1 cargo run --bin crawler
REGION=eu-west-1 cargo run --bin crawler
REGION=ap-south-1 cargo run --bin crawler
```

## Observability

Extended `/api/health/frontier` response:
```json
{
  "global_pending": 1500000,
  "regions": [
    {
      "region": "us-east-1",
      "pending": 500000,
      "active_workers": 12,
      "throughput_rps": 45.2
    },
    {
      "region": "eu-west-1",
      "pending": 600000,
      "active_workers": 8,
      "throughput_rps": 32.1
    }
  ]
}
```

## Cost Optimization

1. **Regional Caching**: Use local PostgreSQL replicas for read-heavy queries
2. **S3 Intelligent Tiering**: Move old WARCs to Glacier
3. **Worker Auto-Scaling**: Scale workers based on regional queue depth
4. **Cross-Region Replication**: Only for critical metadata, not all WARCs

## Migration Path

1. **Phase 1**: Add region columns to existing tables
2. **Phase 2**: Deploy CockroachDB alongside existing Postgres
3. **Phase 3**: Migrate frontier data with dual-write pattern
4. **Phase 4**: Cut over to CockroachDB as primary
5. **Phase 5**: Deploy regional worker pools
