# ArchiveStream Public API (v1)

Welcome to the ArchiveStream Public API. This API provides programmatic access to the archive's metadata, search indices, and temporal differentials.

## 📍 Base URL
`http://localhost:3001/api/v1`

---

## 📸 Snapshots API

### Find Snapshots
`GET /snapshots`

Returns a list of snapshots for a specific URL.

**Query Parameters:**
| Parameter | Type | Required | Description |
| :--- | :--- | :--- | :--- |
| `url` | string | Yes | The canonical URL to search for. |
| `limit` | integer | No | Max results to return (default: 50). |

**Example:**
`GET /api/v1/snapshots?url=https://google.com`

---

## 🔍 Search API

### Global Search
`GET /search`

Search the full-text index across all archived domains.

**Query Parameters:**
| Parameter | Type | Required | Description |
| :--- | :--- | :--- | :--- |
| `q` | string | Yes | The search query string. |

**Example:**
`GET /api/v1/search?q=privacy+policy`

---

## ⌛ Timeline API

### Get Timeline
`GET /timeline`

Returns a discrete time-series of snapshots for a URL.

**Query Parameters:**
| Parameter | Type | Required | Description |
| :--- | :--- | :--- | :--- |
| `url` | string | Yes | The canonical URL. |

---

## 🎯 Resolve API

### Resolve Best Snapshot
`GET /resolve`

Finds the closest snapshot to a requested point in time.

**Query Parameters:**
| Parameter | Type | Required | Description |
| :--- | :--- | :--- | :--- |
| `url` | string | Yes | The canonical URL. |
| `at` | string | Yes | Timestamp in `YYYYMMDDHHMMSS` format. |

**Response Schema:**
```json
{
  "requested_at": "20200101000000",
  "actual_timestamp": "20191231235959",
  "replay_url": "/web/20191231235959/https://example.com"
}
```

---

## 📉 Diff API

### Compute Differential
`GET /diff`

Computes a semantic text diff between two snapshots.

**Query Parameters:**
| Parameter | Type | Required | Description |
| :--- | :--- | :--- | :--- |
| `url` | string | Yes | The canonical URL. |
| `from` | string | Yes | Base timestamp (`YYYYMMDDHHMMSS`). |
| `to` | string | Yes | Target timestamp (`YYYYMMDDHHMMSS`). |

---

## 🛡️ Rate Limits & Auth
- **Public Access**: 100 requests / minute per IP.
- **API Keys**: Required for higher throughput (Coming soon in v1.1).
