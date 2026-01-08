# Horizontal Crawler Scaling

ArchiveStream uses a **stateless crawler architecture** coordinated by a database-backed **URL Frontier**. This allows multiple crawler nodes to run in parallel without coordination overhead.

## 🚀 The URL Frontier

The frontier is a PostgreSQL table that manages the lifecycle of a URL from discovery to archival.

### `url_frontier` Schema
- `url` (TEXT PRIMARY KEY): Canonical URL.
- `domain` (TEXT): For domain-level politeness.
- `priority` (INT): Higher numbers crawled first (default: 0).
- `next_fetch_at` (TIMESTAMPTZ): Scheduled crawl time (supports backoff).
- `fetch_attempts` (INT): Number of failed attempts.
- `leased_until` (TIMESTAMPTZ): Lock for horizontal scaling.
- `last_status` (INT): Last HTTP status (optional).
- `created_at` (TIMESTAMPTZ).

## 🧬 Claiming Logic (Atomic)

To ensure no two crawlers fetch the same URL at the same time, we use a single atomic `UPDATE` with `SKIP LOCKED` or lease-based claiming.

```sql
UPDATE url_frontier
SET leased_until = now() + interval '1 minute'
WHERE url IN (
    SELECT url FROM url_frontier
    WHERE (leased_until IS NULL OR leased_until < now())
      AND next_fetch_at <= now()
    ORDER BY priority DESC, created_at ASC
    LIMIT 10
    FOR UPDATE SKIP LOCKED
)
RETURNING *;
```

## 🚦 Politeness Enforcement

Crawlers must respect `robots.txt` and maintain a minimum delay between requests to the same domain.

1. **Wait time**: Minimum 1 second between requests to the same domain (configurable).
2. **Lease expiry**: If a crawler crashes, the lease expires, and the URL is automatically returned to the frontier.
3. **Backoff**: 4xx/5xx errors trigger an exponential backoff by updating `next_fetch_at`.
