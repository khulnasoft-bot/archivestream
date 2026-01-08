mod opensearch_client;

use archive_common::extractor::extract_text;

use opensearch_client::SearchClient;
use sqlx::PgPool;
use std::sync::Arc;
use tracing::{info, error};
use archive_common::Snapshot;
use serde_json::json;
use url::Url;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://admin:password@localhost/archivestream".into());
    let opensearch_url = std::env::var("OPENSEARCH_URL")
        .unwrap_or_else(|_| "http://localhost:9200".into());

    let pool = PgPool::connect(&database_url).await?;
    let search_client = SearchClient::new(&opensearch_url)?;

    search_client.ensure_index().await?;
    info!("Indexer started, connected to DB and OpenSearch");

    loop {
        match process_pending_snapshots(&pool, &search_client).await {
            Ok(count) if count > 0 => info!("Indexed {} snapshots", count),
            Err(e) => error!("Indexer error: {}", e),
            _ => (),
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
    }
}

async fn process_pending_snapshots(pool: &PgPool, search_client: &SearchClient) -> anyhow::Result<usize> {
    // 1. Get unindexed HTML snapshots
    // Note: In real setup, we'd need to add 'indexed_at' column to DB. 
    // For this design, we'll assume it exists or we use a temporary set of IDs.
    let snapshots = sqlx::query_as!(
        Snapshot,
        r#"
        SELECT s.id, s.url, s.timestamp, 
               COALESCE(p.warc_path, s.warc_file) as "warc_file!",
               COALESCE(p.warc_offset, s.offset) as "offset!",
               s.length as "length!", s.sha256, s.status_code as "status_code!", s.content_type,
               s.payload_hash
        FROM snapshots s
        LEFT JOIN payloads p ON s.payload_hash = p.hash
        WHERE s.content_type LIKE '%html%'
        LIMIT 10
        "#
    )
    .fetch_all(pool)
    .await?;

    let mut docs = Vec::new();
    let count = snapshots.len();

    // In a real implementation, we would load the WARC bytes here. 
    // Since we are simulating, we'll index with mock labels if indexing fails.
    for snapshot in snapshots {
        let domain = Url::parse(&snapshot.url).ok()
            .and_then(|u| u.domain().map(|d| d.to_string()))
            .unwrap_or_default();

        // Simplified for MVP: In real life, fetch bytes from S3 here
        let extracted = extractor::extract_text("<html><title>Sample</title><body>Sample content for indexing</body></html>");

        docs.push(json!({
            "snapshot_id": snapshot.id,
            "url": snapshot.url,
            "domain": domain,
            "timestamp": snapshot.timestamp,
            "title": extracted.title,
            "content": extracted.text_content,
            "mime": snapshot.content_type,
        }));
    }

    search_client.index_snapshots(docs).await?;
    
    // 2. Mark as indexed
    // sqlx::query!("UPDATE snapshots SET indexed_at = NOW() WHERE id = ANY($1)", ids).execute(pool).await?;

    Ok(count)
}
