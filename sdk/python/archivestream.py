import requests
from typing import List, Dict, Optional

class ArchiveStream:
    """
    Official Python SDK for ArchiveStream.
    """
    def __init__(self, base_url: str = "http://localhost:3001"):
        self.base_url = f"{base_url.rstrip('/')}/api/v1"

    def search(self, query: str) -> List[Dict]:
        """Search the archive for a query string."""
        response = requests.get(f"{self.base_url}/search", params={"q": query})
        response.raise_for_status()
        return response.json()

    def get_snapshots(self, url: str, limit: int = 50) -> List[Dict]:
        """Get all snapshots for a specific URL."""
        response = requests.get(f"{self.base_url}/snapshots", params={"url": url, "limit": limit})
        response.raise_for_status()
        return response.json()

    def resolve(self, url: str, at: str) -> Dict:
        """Resolve the best snapshot for a URL at a given time."""
        response = requests.get(f"{self.base_url}/resolve", params={"url": url, "at": at})
        response.raise_for_status()
        return response.json()

    def get_diff(self, url: str, from_ts: str, to_ts: str) -> Dict:
        """Get the semantic diff between two snapshots."""
        response = requests.get(f"{self.base_url}/diff", params={"url": url, "from": from_ts, "to": to_ts})
        response.raise_for_status()
        return response.json()

    def get_semantic(self, url: str, from_ts: str, to_ts: str) -> Dict:
        """Categorize changes between two snapshots."""
        response = requests.get(f"{self.base_url}/semantic", params={"url": url, "from": from_ts, "to": to_ts})
        response.raise_for_status()
        return response.json()

    def get_timeline(self, url: str) -> Dict:
        """Get the full history timeline for a URL."""
        response = requests.get(f"{self.base_url}/timeline", params={"url": url})
        response.raise_for_status()
        return response.json()
