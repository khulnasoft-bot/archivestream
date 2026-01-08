# ArchiveStream Python SDK

The official Python client for the ArchiveStream API.

## Installation
```bash
pip install requests
```

## Usage
```python
from archivestream import ArchiveStream

archive = ArchiveStream("http://localhost:3001")

# Search for content
results = archive.search("privacy policy")

# Get snapshots for a URL
snapshots = archive.get_snapshots("https://example.com")

# Resolve the best snapshot for a point in time
resolved = archive.resolve("https://example.com", "20240101000000")
print(f"Replay at: {resolved['replay_url']}")
```
