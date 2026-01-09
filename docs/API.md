# ArchiveStream API Reference

## Base URL
```
https://api.archivestream.org/api/v1
```

## Authentication
Currently, the public API is rate-limited but does not require authentication. For self-hosted instances, configure API keys via environment variables.

---

## Endpoints

### 1. Search Snapshots
Search across all archived content.

**Endpoint:** `GET /search`

**Query Parameters:**
- `q` (string, required): Search query

**Example Request:**
```bash
curl "https://api.archivestream.org/api/v1/search?q=climate+change"
```

**Example Response:**
```json
{
  "results": [
    {
      "snapshot_id": "123e4567-e89b-12d3-a456-426614174000",
      "url": "https://example.com/article",
      "title": "Climate Change Report 2024",
      "timestamp": "2024-03-15T10:30:00Z",
      "snippet": "...highlights from the report..."
    }
  ],
  "total": 42
}
```

---

### 2. Get Snapshots for URL
Retrieve all snapshots for a specific URL.

**Endpoint:** `GET /snapshots`

**Query Parameters:**
- `url` (string, required): The URL to query
- `limit` (integer, optional): Max results (default: 50)

**Example Request:**
```bash
curl "https://api.archivestream.org/api/v1/snapshots?url=https://example.com&limit=10"
```

**Example Response:**
```json
[
  {
    "id": "123e4567-e89b-12d3-a456-426614174000",
    "url": "https://example.com",
    "timestamp": "2024-03-15T10:30:00Z",
    "status_code": 200,
    "content_type": "text/html"
  }
]
```

---

### 3. Get Timeline
Retrieve a visual timeline of changes for a URL.

**Endpoint:** `GET /timeline`

**Query Parameters:**
- `url` (string, required): The URL to analyze

**Example Request:**
```bash
curl "https://api.archivestream.org/api/v1/timeline?url=https://example.com"
```

**Example Response:**
```json
{
  "url": "https://example.com",
  "snapshots": [
    {
      "timestamp": "2024-01-01T00:00:00Z",
      "status": 200,
      "digest": "abc123...",
      "intensity": 0.1
    },
    {
      "timestamp": "2024-02-01T00:00:00Z",
      "status": 200,
      "digest": "def456...",
      "intensity": 0.9
    }
  ]
}
```

---

### 4. Get Diff
Compare two snapshots of the same URL.

**Endpoint:** `GET /diff`

**Query Parameters:**
- `url` (string, required): The URL
- `from` (string, required): Timestamp in format `YYYYMMDDHHMMSS`
- `to` (string, required): Timestamp in format `YYYYMMDDHHMMSS`

**Example Request:**
```bash
curl "https://api.archivestream.org/api/v1/diff?url=https://example.com&from=20240101000000&to=20240201000000"
```

**Example Response:**
```json
{
  "url": "https://example.com",
  "from_timestamp": "20240101000000",
  "to_timestamp": "20240201000000",
  "added_lines": ["New content here"],
  "removed_lines": ["Old content removed"],
  "semantic_categories": ["content_update", "layout_change"]
}
```

---

### 5. Resolve Snapshot
Find the closest snapshot to a given timestamp.

**Endpoint:** `GET /resolve`

**Query Parameters:**
- `url` (string, required): The URL
- `at` (string, required): Desired timestamp in format `YYYYMMDDHHMMSS`

**Example Request:**
```bash
curl "https://api.archivestream.org/api/v1/resolve?url=https://example.com&at=20240315103000"
```

**Example Response:**
```json
{
  "requested_at": "20240315103000",
  "actual_timestamp": "20240315102847",
  "replay_url": "/web/20240315102847/https://example.com"
}
```

---

### 6. Trigger Crawl
Queue a URL for archiving.

**Endpoint:** `POST /crawl` (Legacy, use federation for production)

**Request Body:**
```json
{
  "url": "https://example.com"
}
```

**Example Request:**
```bash
curl -X POST https://api.archivestream.org/crawl \
  -H "Content-Type: application/json" \
  -d '{"url":"https://example.com"}'
```

**Example Response:**
```json
{
  "status": "queued",
  "url": "https://example.com"
}
```

---

## Rate Limits
- **Public API**: 100 requests/minute per IP
- **Self-hosted**: Configurable via `RATE_LIMIT_RPM` environment variable

## SDKs
- **Python**: `pip install archivestream`
- **JavaScript**: `npm install @archivestream/client`
- **Go**: Community-maintained at `github.com/archivestream/go-client`

## Support
- Documentation: https://docs.archivestream.org
- Discord: https://discord.gg/archivestream
- GitHub Issues: https://github.com/archivestream/archivestream/issues
