mod replay;
mod search;
mod diff;
mod semantic;
mod federation;

use axum::{
    extract::{Query, Path, State},
    routing::{get, post},
    Json, Router,
    response::{IntoResponse, Response},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use archive_common::Snapshot;
use archive_common::replay::ReplayUrl;
use uuid::Uuid;
use chrono::Utc;
use sqlx::PgPool;
use crate::replay::{Resolver, WarcReader, Rewriter};
use crate::search::SearchService;
use archive_federation::PeerManager;
use opensearch::http::transport::Transport;
use opensearch::OpenSearch;

pub struct AppConfig {
    pub node_id: String,
}

pub struct AppState {
    pub pool: PgPool,
    pub resolver: Resolver,
    pub warc_reader: WarcReader,
    pub search_service: SearchService,
    pub peer_manager: PeerManager,
    pub intelligence_engine: Arc<dyn archive_intelligence::IntelligenceEngine>,
    pub notification_dispatcher: Arc<dyn archive_notification::NotificationDispatcher>,
    pub config: AppConfig,
}

#[derive(Deserialize)]
struct GlobalSearchQuery {
    q: String,
}

#[derive(Deserialize)]
struct TimelineQuery {
    url: String,
}

#[derive(Serialize)]
struct TimelineResponse {
    url: String,
    snapshots: Vec<TimelineSnapshot>,
}

#[derive(Serialize)]
struct TimelineSnapshot {
    timestamp: chrono::DateTime<Utc>,
    status: u16,
    digest: String,
    intensity: f32, // Heatmap intensity 0.0 to 1.0
}

#[derive(Deserialize)]
struct DiffQuery {
    url: String,
    from: String, // timestamp string format YYYYMMDDHHMMSS
    to: String,
}

#[derive(Serialize)]
struct FrontierHealth {
    domain: String,
    count: i64,
    depth_range: (i32, i32),
}

#[derive(Serialize)]
struct OutcomeMetric {
    status: String,
    count: i64,
}

// API v1 Structs
#[derive(Deserialize)]
struct SnapshotsQuery {
    url: String,
    #[allow(dead_code)]
    pub from: Option<String>,
    #[allow(dead_code)]
    pub to: Option<String>,
    limit: Option<i64>,
}

#[derive(Deserialize)]
struct ResolveQuery {
    url: String,
    at: String, // timestamp string format YYYYMMDDHHMMSS
}

#[derive(Serialize)]
struct ResolveResponse {
    requested_at: String,
    actual_timestamp: String,
    replay_url: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "archive_api=info,tower_http=debug".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://admin:password@localhost/archivestream".into());
    let pool = PgPool::connect(&database_url).await?;

    let s3_endpoint = std::env::var("S3_ENDPOINT").unwrap_or_else(|_| "http://localhost:9000".into());
    let opensearch_url = std::env::var("OPENSEARCH_URL").unwrap_or_else(|_| "http://localhost:9200".into());
    
    let transport = Transport::single_node(&opensearch_url)?;
    let os_client = OpenSearch::new(transport);

    let node_id = std::env::var("NODE_ID").unwrap_or_else(|_| Uuid::new_v4().to_string());
    
    // Initialize Intelligence Engine (Phase 7)
    let openai_key = std::env::var("OPENAI_API_KEY").ok();
    let llm_engine = openai_key.map(|key| {
        let model = std::env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-4-turbo-preview".to_string());
        archive_intelligence::LLMIntelligenceEngine::new(key, model, None)
    });
    let intelligence_engine: Arc<dyn archive_intelligence::IntelligenceEngine> = Arc::new(
        archive_intelligence::HybridEngine::new(llm_engine)
    );

    let notification_dispatcher: Arc<dyn archive_notification::NotificationDispatcher> = Arc::new(
        archive_notification::MultiChannelDispatcher::new()
    );

    let state = Arc::new(AppState {
        pool: pool.clone(),
        resolver: Resolver::new(pool),
        warc_reader: WarcReader::new(s3_endpoint),
        search_service: SearchService::new(os_client),
        peer_manager: PeerManager::new(node_id.clone()),
        intelligence_engine,
        notification_dispatcher,
        config: AppConfig { node_id },
    });

    let v1_api = Router::new()
        .route("/snapshots", get(get_snapshots_v1))
        .route("/search", get(global_search))
        .route("/timeline", get(get_timeline))
        .route("/resolve", get(resolve_v1))
        .route("/semantic", get(semantic::get_semantic_change))
        .route("/diff", get(get_diff))
        .route("/federation/peers", get(federation::get_peers))
        .route("/federation/search", get(federation::search_federated))
        .route("/federation/manifest", get(federation::get_manifest))
        .route("/federation/handshake", post(federation::handle_handshake));

    let app = Router::new()
        .route("/", get(|| async { "ArchiveStream API v0.1.0" }))
        .route("/health", get(|| async { Json(serde_json::json!({"status": "ok"})) }))
        .nest("/api/v1", v1_api)
        // Legacy routes (optional, keep for UI for now)
        .route("/search", get(global_search))
        .route("/timeline", get(get_timeline))
        .route("/diff", get(get_diff))
        .route("/health/frontier", get(get_frontier_health))
        .route("/health/outcomes", get(get_outcomes))
        .route("/snapshots", get(get_snapshots_v1))
        .route("/snapshot/:id", get(get_snapshot))
        .route("/snapshot/:id/download", get(federation::download_snapshot))
        .route("/crawl", post(start_crawl))
        .route("/web/:timestamp/*url", get(replay_handler))
        .with_state(state.clone());

    // Spawn Federation Sync Worker
    let sync_state = state.clone();
    tokio::spawn(async move {
        let worker = federation::SyncWorker::new(sync_state);
        worker.run_loop().await;
    });

    let addr = SocketAddr::from(([0, 0, 0, 0], 3001));
    tracing::info!("listening on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn get_timeline(
    State(state): State<Arc<AppState>>,
    Query(params): Query<TimelineQuery>,
) -> impl IntoResponse {
    let rows = sqlx::query(
        r#"
        SELECT timestamp, status_code, sha256
        FROM snapshots
        WHERE url = $1
        ORDER BY timestamp ASC
        "#
    )
    .bind(&params.url)
    .fetch_all(&state.pool)
    .await;

    match rows {
        Ok(rows) => {
            use sqlx::Row;
            let mut last_digest = String::new();
            let snapshots = rows.into_iter().map(|row| {
                let status_code: i16 = row.get("status_code");
                let digest: String = row.get("sha256");
                let intensity = if digest != last_digest && !last_digest.is_empty() { 1.0 } else { 0.1 };
                last_digest = digest.clone();
                
                TimelineSnapshot {
                    timestamp: row.get("timestamp"),
                    status: status_code as u16,
                    digest,
                    intensity,
                }
            }).collect::<Vec<_>>();
            
            Json(TimelineResponse {
                url: params.url,
                snapshots,
            }).into_response()
        }
        Err(e) => {
            tracing::error!("Timeline error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Timeline failed").into_response()
        }
    }
}

async fn get_diff(
    State(state): State<Arc<AppState>>,
    Query(params): Query<DiffQuery>,
) -> impl IntoResponse {
    use crate::diff::DiffService;
    use archive_common::replay::ReplayUrl;

    // Resolve snapshots for both timestamps
    let ts_from = match ReplayUrl::parse(&params.from, &params.url) {
        Ok(u) => u.timestamp,
        Err(_) => return (StatusCode::BAD_REQUEST, "Invalid FROM timestamp").into_response(),
    };
    let ts_to = match ReplayUrl::parse(&params.to, &params.url) {
        Ok(u) => u.timestamp,
        Err(_) => return (StatusCode::BAD_REQUEST, "Invalid TO timestamp").into_response(),
    };

    let s1 = state.resolver.resolve(&params.url, ts_from).await.ok().flatten();
    let s2 = state.resolver.resolve(&params.url, ts_to).await.ok().flatten();

    if s1.is_none() || s2.is_none() {
        return (StatusCode::NOT_FOUND, "One or both snapshots not found").into_response();
    }

    let s1 = s1.unwrap();
    let s2 = s2.unwrap();

    let d1 = state.warc_reader.read_record(&s1.warc_file, s1.offset, s1.length).await.unwrap_or_default();
    let d2 = state.warc_reader.read_record(&s2.warc_file, s2.offset, s2.length).await.unwrap_or_default();

    let h1 = String::from_utf8_lossy(&d1);
    let h2 = String::from_utf8_lossy(&d2);

    let diff = DiffService::compute_diff(&h1, &h2, &params.from, &params.to);

    Json(diff).into_response()
}

async fn get_frontier_health(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let rows = sqlx::query(
        r#"
        SELECT domain, COUNT(*) as count, MIN(depth) as min_depth, MAX(depth) as max_depth
        FROM url_frontier
        GROUP BY domain
        ORDER BY count DESC
        LIMIT 50
        "#
    )
    .fetch_all(&state.pool)
    .await;

    match rows {
        Ok(rows) => {
            use sqlx::Row;
            let health = rows.into_iter().map(|row| {
                let count: i64 = row.get("count");
                let min_depth: i32 = row.get("min_depth");
                let max_depth: i32 = row.get("max_depth");
                FrontierHealth {
                    domain: row.get("domain"),
                    count,
                    depth_range: (min_depth, max_depth),
                }
            }).collect::<Vec<_>>();
            Json(health).into_response()
        }
        Err(e) => {
            tracing::error!("Health error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Health failed").into_response()
        }
    }
}

async fn get_outcomes(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let rows = sqlx::query(
        r#"
        SELECT status, COUNT(*) as count
        FROM crawl_events
        WHERE timestamp > NOW() - interval '24 hours'
        GROUP BY status
        "#
    )
    .fetch_all(&state.pool)
    .await;

    match rows {
        Ok(rows) => {
            use sqlx::Row;
            let metrics = rows.into_iter().map(|row| OutcomeMetric {
                status: row.get("status"),
                count: row.get("count"),
            }).collect::<Vec<_>>();
            Json(metrics).into_response()
        }
        Err(e) => {
            tracing::error!("Outcomes error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Outcomes failed").into_response()
        }
    }
}

async fn replay_handler(
    State(state): State<Arc<AppState>>,
    Path((timestamp_str, url_str)): Path<(String, String)>,
) -> impl IntoResponse {
    let replay_url = match ReplayUrl::parse(&timestamp_str, &url_str) {
        Ok(u) => u,
        Err(_) => return (StatusCode::BAD_REQUEST, "Invalid timestamp or URL").into_response(),
    };

    let snapshot = match state.resolver.resolve(&replay_url.original_url, replay_url.timestamp).await {
        Ok(Some(s)) => s,
        Ok(None) => return (StatusCode::NOT_FOUND, "Snapshot not found").into_response(),
        Err(e) => {
            tracing::error!("DB Error: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response();
        }
    };

    let raw_data = match state.warc_reader.read_record(&snapshot.warc_file, snapshot.offset, snapshot.length).await {
        Ok(d) => d,
        Err(e) => {
            tracing::error!("Storage Error: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Error reading archive").into_response();
        }
    };

    // Replay Logic: Rewrite if HTML
    if snapshot.content_type.contains("html") {
        let rewriter = Rewriter::new(snapshot.timestamp, snapshot.url);
        let rewritten_body = rewriter.rewrite_html(&raw_data);
        
        Response::builder()
            .status(StatusCode::from_u16(snapshot.status_code as u16).unwrap_or(StatusCode::OK))
            .header("Content-Type", "text/html")
            .body(axum::body::Body::from(rewritten_body))
            .unwrap()
            .into_response()
    } else {
        Response::builder()
            .status(StatusCode::from_u16(snapshot.status_code as u16).unwrap_or(StatusCode::OK))
            .header("Content-Type", snapshot.content_type)
            .body(axum::body::Body::from(raw_data))
            .unwrap()
            .into_response()
    }
}

async fn global_search(
    State(state): State<Arc<AppState>>,
    Query(params): Query<GlobalSearchQuery>,
) -> impl IntoResponse {
    match state.search_service.search(&params.q).await {
        Ok(results) => Json(results).into_response(),
        Err(e) => {
            tracing::error!("Search error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Search failed").into_response()
        }
    }
}

async fn resolve_v1(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ResolveQuery>,
) -> impl IntoResponse {
    let ts = match ReplayUrl::parse(&params.at, &params.url) {
        Ok(u) => u.timestamp,
        Err(_) => return Response::builder().status(400).body("Invalid timestamp format".into()).unwrap(),
    };

    match state.resolver.resolve(&params.url, ts).await {
        Ok(Some(s)) => {
            let actual_ts = s.timestamp.format("%Y%m%d%H%M%S").to_string();
            Json(ResolveResponse {
                requested_at: params.at,
                actual_timestamp: actual_ts.clone(),
                replay_url: format!("/web/{}/{}", actual_ts, params.url),
            }).into_response()
        }
        Ok(None) => (StatusCode::NOT_FOUND, "No snapshot found for given URL and time").into_response(),
        Err(e) => {
            tracing::error!("Resolve error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Internal error").into_response()
        }
    }
}

async fn get_snapshots_v1(
    State(state): State<Arc<AppState>>,
    Query(params): Query<SnapshotsQuery>,
) -> impl IntoResponse {
    let limit = params.limit.unwrap_or(50);
    
    // Simple query: in v2 we would add date range filters
    // Simple query: in v2 we would add date range filters
    let result = sqlx::query_as::<_, Snapshot>(
        r#"
        SELECT id, url, timestamp, warc_file, offset, length, sha256, status_code, content_type, payload_hash
        FROM snapshots
        WHERE url = $1
        ORDER BY timestamp DESC
        LIMIT $2
        "#
    )
    .bind(&params.url)
    .bind(limit)
    .fetch_all(&state.pool)
    .await;

    match result {
        Ok(snapshots) => Json(snapshots).into_response(),
        Err(e) => {
            tracing::error!("Snapshots error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Error fetching snapshots").into_response()
        }
    }
}

async fn get_snapshot(Path(id): Path<Uuid>, State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let result = sqlx::query_as::<_, Snapshot>(
        r#"
        SELECT id, url, timestamp, warc_file, offset, length, sha256, status_code, content_type, payload_hash
        FROM snapshots
        WHERE id = $1
        "#
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await;

    match result {
        Ok(Some(s)) => Json(s).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, "Snapshot not found").into_response(),
        Err(e) => {
            tracing::error!("Snapshot error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Error fetching snapshot").into_response()
        }
    }
}

async fn start_crawl(State(_state): State<Arc<AppState>>, Json(payload): Json<serde_json::Value>) -> Json<serde_json::Value> {
    // Basic crawl trigger (placeholder logic)
    let url = payload.get("url").and_then(|v| v.as_str()).unwrap_or("");
    tracing::info!("Starting crawl for: {}", url);
    Json(serde_json::json!({"status": "queued", "url": url}))
}
