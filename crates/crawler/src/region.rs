use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Regions supported by ArchiveStream
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Region {
    UsEast1,
    EuWest1,
    ApSouth1,
}

impl Region {
    pub fn as_str(&self) -> &'static str {
        match self {
            Region::UsEast1 => "us-east-1",
            Region::EuWest1 => "eu-west-1",
            Region::ApSouth1 => "ap-south-1",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "us-east-1" => Some(Region::UsEast1),
            "eu-west-1" => Some(Region::EuWest1),
            "ap-south-1" => Some(Region::ApSouth1),
            _ => None,
        }
    }

    pub fn from_env() -> Self {
        std::env::var("REGION")
            .ok()
            .and_then(|r| Self::from_str(&r))
            .unwrap_or(Region::UsEast1)
    }
}

/// Consistent hashing for domain -> region affinity
pub struct RegionRouter {
    regions: Vec<Region>,
}

impl RegionRouter {
    pub fn new() -> Self {
        Self {
            regions: vec![Region::UsEast1, Region::EuWest1, Region::ApSouth1],
        }
    }

    /// Assign a domain to a preferred region using consistent hashing
    pub fn route_domain(&self, domain: &str) -> Region {
        let mut hasher = DefaultHasher::new();
        domain.hash(&mut hasher);
        let hash = hasher.finish();
        
        let index = (hash as usize) % self.regions.len();
        self.regions[index].clone()
    }

    /// Check if this worker should prioritize a domain based on region affinity
    pub fn should_prioritize(&self, domain: &str, worker_region: &Region) -> bool {
        &self.route_domain(domain) == worker_region
    }
}

impl Default for RegionRouter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consistent_routing() {
        let router = RegionRouter::new();
        
        // Same domain should always route to same region
        let domain = "example.com";
        let region1 = router.route_domain(domain);
        let region2 = router.route_domain(domain);
        assert_eq!(region1, region2);
    }

    #[test]
    fn test_distribution() {
        let router = RegionRouter::new();
        let domains: Vec<String> = (0..1000).map(|i| format!("domain{}.com", i)).collect();
        
        let mut counts = std::collections::HashMap::new();
        for domain in domains {
            let region = router.route_domain(&domain);
            *counts.entry(region.as_str()).or_insert(0) += 1;
        }
        
        // Rough distribution check (should be ~333 each)
        for count in counts.values() {
            assert!(*count > 250 && *count < 400, "Distribution skewed: {}", count);
        }
    }
}
