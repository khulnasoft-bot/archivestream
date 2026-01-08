# Storage Model

ArchiveStream uses a two-tier storage model: Immutable WARC files for raw data and a PostgreSQL metadata index for fast retrieval and deduplication.

## 🗄️ Database Schema

### `snapshots`
Tracks every unique crawl event.
- `id` (UUID): Primary key.
- `url` (TEXT): The original target URL.
- `timestamp` (TIMESTAMPTZ): When the page was crawled.
- `warc_file` (TEXT): Path to the WARC file containing this record (if not deduplicated).
- `offset` (BIGINT): Byte offset within the WARC file.
- `length` (BIGINT): Length of the record in bytes.
- `sha256` (TEXT): Hash of the full HTTP response.
- `status_code` (INT): HTTP status returned.
- `content_type` (TEXT): MIME type.
- `payload_hash` (TEXT): SHA-256 hash of the response body, used for deduplication.

### `payloads`
Tracks unique content to enable deduplication via WARC `revisit` records.
- `hash` (TEXT): SHA-256 of the response body. Primary key.
- `warc_path` (TEXT): The WARC file containing the **first** occurrence of this payload.
- `warc_offset` (BIGINT): Offset to the `response` record in the original WARC.
- `size` (BIGINT): Size of the payload bytes.
- `created_at` (TIMESTAMPTZ): When this payload was first seen.

## 📦 WARC Strategy

ArchiveStream strictly follows the ISO WARC standard.

1. **`response` records**: Stored when a new unique payload is encountered.
2. **`revisit` records**: Stored for duplicate content. These records contain full HTTP headers but omit the payload body, instead referencing the original `response` record via the `WARC-Payload-Digest`.

## 🔄 Replay Resolution

When a replay is requested for `/web/{timestamp}/{url}`:
1. Find the latest `snapshot` for `url` where `timestamp <= requested`.
2. If `snapshot.payload_hash` is present, join with the `payloads` table to find the byte range of the original content.
3. Stream the bytes from S3/MinIO starting at the resolved `warc_offset`.
