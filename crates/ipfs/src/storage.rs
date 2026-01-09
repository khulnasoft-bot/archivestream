use anyhow::Result;
use ipfs_api_backend_hyper::{IpfsApi, IpfsClient, TryFromUri};
use serde::{Deserialize, Serialize};
use std::io::Cursor;
use tracing::{info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpfsSnapshot {
    pub cid: String,
    pub url: String,
    pub timestamp: String,
    pub size: u64,
}

pub struct IpfsStorage {
    client: IpfsClient,
}

impl IpfsStorage {
    /// Connect to IPFS daemon
    pub fn new(api_url: &str) -> Result<Self> {
        let client = IpfsClient::from_str(api_url)?;
        Ok(Self { client })
    }

    /// Store WARC content on IPFS
    pub async fn store_warc(&self, content: &[u8]) -> Result<String> {
        info!("Storing {} bytes to IPFS", content.len());
        
        let cursor = Cursor::new(content);
        let response = self.client.add(cursor).await?;
        
        let cid = response.hash;
        info!("Stored to IPFS with CID: {}", cid);
        
        Ok(cid)
    }

    /// Retrieve WARC content from IPFS
    pub async fn retrieve_warc(&self, cid: &str) -> Result<Vec<u8>> {
        info!("Retrieving CID: {}", cid);
        
        let data = self.client
            .cat(cid)
            .map_ok(|chunk| chunk.to_vec())
            .try_concat()
            .await?;
        
        info!("Retrieved {} bytes from IPFS", data.len());
        Ok(data)
    }

    /// Pin content to ensure persistence
    pub async fn pin(&self, cid: &str) -> Result<()> {
        info!("Pinning CID: {}", cid);
        self.client.pin_add(cid, true).await?;
        Ok(())
    }

    /// Unpin content to allow garbage collection
    pub async fn unpin(&self, cid: &str) -> Result<()> {
        info!("Unpinning CID: {}", cid);
        self.client.pin_rm(cid, true).await?;
        Ok(())
    }

    /// Publish snapshot manifest to IPNS
    pub async fn publish_manifest(&self, manifest: &SnapshotManifest) -> Result<String> {
        let json = serde_json::to_vec(manifest)?;
        let cid = self.store_warc(&json).await?;
        
        // Publish to IPNS (requires IPFS key)
        let response = self.client.name_publish(&cid, true, None, None, None).await?;
        
        Ok(response.name)
    }

    /// Resolve IPNS name to latest CID
    pub async fn resolve_ipns(&self, name: &str) -> Result<String> {
        let response = self.client.name_resolve(Some(name), true, false).await?;
        Ok(response.path)
    }

    /// Get IPFS node stats
    pub async fn stats(&self) -> Result<IpfsStats> {
        let repo_stats = self.client.stats_repo().await?;
        
        Ok(IpfsStats {
            num_objects: repo_stats.num_objects,
            repo_size: repo_stats.repo_size,
            storage_max: repo_stats.storage_max,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SnapshotManifest {
    pub version: String,
    pub snapshots: Vec<IpfsSnapshot>,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IpfsStats {
    pub num_objects: u64,
    pub repo_size: u64,
    pub storage_max: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires running IPFS daemon
    async fn test_ipfs_storage() {
        let storage = IpfsStorage::new("http://localhost:5001").unwrap();
        
        let content = b"Hello, IPFS!";
        let cid = storage.store_warc(content).await.unwrap();
        
        let retrieved = storage.retrieve_warc(&cid).await.unwrap();
        assert_eq!(content, retrieved.as_slice());
    }
}
