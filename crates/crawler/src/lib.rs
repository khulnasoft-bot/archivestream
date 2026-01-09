pub mod fetcher;
pub mod parser;
pub mod robots;
pub mod warc;
pub mod dedup;
pub mod frontier;
pub mod region;
pub mod rate_limit;

use tracing::{info, error};
use crate::fetcher::Fetcher;
use crate::dedup::DedupService;
use crate::frontier::FrontierService;
use crate::region::{Region, RegionRouter};
use crate::rate_limit::RateLimiter;
use sqlx::PgPool;
use archive_intelligence::{PredictiveEngine, StandardPredictor};
use std::sync::Arc;

pub struct Crawler {
    fetcher: Fetcher,
    dedup: DedupService,
    frontier: FrontierService,
    predictor: Arc<dyn PredictiveEngine>,
    #[allow(dead_code)]
    region: Region,
    #[allow(dead_code)]
    region_router: RegionRouter,
    #[allow(dead_code)]
    rate_limiter: RateLimiter,
}

impl Crawler {
    pub fn new(pool: PgPool) -> Self {
        let region = Region::from_env();
        info!("Initializing crawler in region: {}", region.as_str());
        
        Self {
            fetcher: Fetcher::new(),
            dedup: DedupService::new(pool.clone()),
            frontier: FrontierService::new(pool.clone()),
            predictor: Arc::new(StandardPredictor),
            region,
            region_router: RegionRouter::new(),
            rate_limiter: RateLimiter::new(pool),
        }
    }

    pub async fn add_url(&self, url: &str) -> anyhow::Result<()> {
        self.frontier.add_url(url, 0, 0).await
    }

    pub async fn run(&self) -> anyhow::Result<()> {
        info!("Crawler loop started in stateless mode");
        loop {
            // Claim a batch of URLs from the frontier
            match self.frontier.claim_urls(10).await {
                Ok(urls) if !urls.is_empty() => {
                    for f_url in urls {
                        let url = f_url.url;
                        let depth = f_url.depth;
                        info!("Crawling: {} (Depth: {})", url, depth);
                        
                        let start_time = std::time::Instant::now();
                        match self.fetcher.fetch(&url).await {
                            Ok(record) => {
                                let duration = start_time.elapsed().as_millis() as i32;
                                info!("Fetched {} (Digest: {})", url, record.payload_digest);
                                
                                let _ = self.frontier.track_event(&url, "success", Some(record.status_code as i32), duration).await;

                                let is_duplicate = self.dedup.is_duplicate(&record.payload_digest).await.unwrap_or(false);
                                
                                if is_duplicate {
                                    info!("Deduplicated: Payload exists for {}", url);
                                    let _warc = record.to_warc_string(true);
                                    // TODO: Save to S3
                                } else {
                                    info!("New payload for {}", url);
                                    let _warc = record.to_warc_string(false);
                                    // TODO: Save to S3
                                    let _ = self.dedup.insert_payload(
                                        &record.payload_digest,
                                        "shared.warc.gz",
                                        0,
                                        record.content.len() as u64
                                    ).await;
                                }

                                // Mark as complete in frontier
                                // Mark as complete in frontier OR reschedule if it's a known URL (Phase 7.3)
                                let history = self.frontier.get_snapshot_history(&url).await.unwrap_or_default();
                                if !history.is_empty() {
                                    match self.predictor.predict_next_crawl(&history).await {
                                        Ok(prediction) => {
                                            info!("Rescheduling {}: Next crawl at {}, Priority: {}", url, prediction.next_fetch_at, prediction.recommended_priority);
                                            let _ = self.frontier.reschedule(&url, prediction.next_fetch_at, prediction.recommended_priority).await;
                                        }
                                        Err(_) => {
                                            let _ = self.frontier.complete(&url).await;
                                        }
                                    }
                                } else {
                                    let _ = self.frontier.complete(&url).await;
                                }


                                // Extract links and feed back to frontier
                                if record.content_type.contains("html") {
                                    let html = String::from_utf8_lossy(&record.content);
                                    let links = parser::extract_links(&url, &html);
                                    info!("Discovered {} links", links.len());
                                    
                                    for link in links {
                                        let _ = self.frontier.add_url(&link, 0, depth + 1).await;
                                    }
                                }
                            }
                            Err(e) => {
                                let duration = start_time.elapsed().as_millis() as i32;
                                error!("Failed {}: {}. Exponential backoff applied.", url, e);
                                let _ = self.frontier.track_event(&url, "error", None, duration).await;
                                let _ = self.frontier.fail(&url, 3600).await; // 1 hour backoff
                            }
                        }
                    }
                }
                Ok(_) => {
                    // No URLs to claim, wait
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                }
                Err(e) => {
                    error!("Frontier claim error: {}", e);
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                }
            }
        }
    }
}
