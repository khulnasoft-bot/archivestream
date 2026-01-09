pub mod storage;

pub use storage::{IpfsStorage, IpfsSnapshot, SnapshotManifest, IpfsStats};

use sha2::{Sha256, Digest};

/// Generate content identifier (CID) for data
pub fn generate_cid(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let hash = hasher.finalize();
    
    // Simplified CID - in production use multihash + multibase
    format!("Qm{}", bs58::encode(&hash).into_string())
}

/// Verify data matches CID
pub fn verify_cid(data: &[u8], cid: &str) -> bool {
    let computed = generate_cid(data);
    computed == cid
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cid_generation() {
        let data = b"test data";
        let cid = generate_cid(data);
        assert!(cid.starts_with("Qm"));
        assert!(verify_cid(data, &cid));
    }

    #[test]
    fn test_cid_verification() {
        let data = b"test data";
        let cid = generate_cid(data);
        
        assert!(verify_cid(data, &cid));
        assert!(!verify_cid(b"different data", &cid));
    }
}
