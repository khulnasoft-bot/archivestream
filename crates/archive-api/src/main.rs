mod replay;
mod search;
mod diff;
mod semantic;

use axum::{
    extract::{Query, Path, State},
    routing::{get, post},
    Json, Router,
    response::{IntoResponse, Response},
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
use opensearch::http::transport::Transport;
use opensearch::OpenSearch;

struct AppState {
    pool: PgPool,
    resolver: Resolver,
    warc_reader: WarcReader,
    search_service: SearchService,
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
    from: Option<String>,
    to: Option<String>,
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
async fn main() -> anyhow::Result<()> {
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

    let state = Arc::new(AppState {
        pool: pool.clone(),
        resolver: Resolver::new(pool),
        warc_reader: WarcReader::new(s3_endpoint),
        search_service: SearchService::new(os_client),
    });

    let v1_api = Router::new()
        .route("/snapshots", get(get_snapshots_v1))
        .route("/search", get(global_search))
        .route("/timeline", get(get_timeline))
        .route("/resolve", get(resolve_v1))
        .route("/semantic", get(semantic::get_semantic_change))
        .route("/diff", get(get_diff));

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
        .route("/crawl", post(start_crawl))
        .route("/web/:timestamp/*url", get(replay_handler))
        .with_state(state);

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
    let result = sqlx::query!(
        r#"
        SELECT timestamp, status_code as "status_code!", sha256
        FROM snapshots
        WHERE url = $1
        ORDER BY timestamp ASC
        "#,
        params.url
    )
    .fetch_all(&state.pool)
    .await;

    match result {
        Ok(rows) => {
            let snapshots = rows.into_iter().map(|row| TimelineSnapshot {
                timestamp: row.timestamp,
                status: row.status_code as u16,
                digest: row.sha256,
            }).collect();
            
            Json(TimelineResponse {
                url: params.url,
                snapshots,
            }).into_response()
        }
        Err(e) => {
            tracing::error!("Timeline error: {}", e);
            Response::builder().status(500).body("Timeline failed".into()).unwrap()
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
        Err(_) => return Response::builder().status(400).body("Invalid FROM timestamp".into()).unwrap(),
    };
    let ts_to = match ReplayUrl::parse(&params.to, &params.url) {
        Ok(u) => u.timestamp,
        Err(_) => return Response::builder().status(400).body("Invalid TO timestamp".into()).unwrap(),
    };

    let s1 = state.resolver.resolve(&params.url, ts_from).await.ok().flatten();
    let s2 = state.resolver.resolve(&params.url, ts_to).await.ok().flatten();

    if s1.is_none() || s2.is_none() {
        return Response::builder().status(404).body("One or both snapshots not found".into()).unwrap();
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
    let result = sqlx::query!(
        r#"
        SELECT domain as "domain!", COUNT(*) as "count!", MIN(depth) as "min_depth!", MAX(depth) as "max_depth!"
        FROM url_frontier
        GROUP BY domain
        ORDER BY count DESC
        LIMIT 50
        "#
    )
    .fetch_all(&state.pool)
    .await;

    match result {
        Ok(rows) => {
            let health = rows.into_iter().map(|row| FrontierHealth {
                domain: row.domain,
                count: row.count,
                depth_range: (row.min_depth, row.max_depth),
            }).collect::<Vec<_>>();
            Json(health).into_response()
        }
        Err(e) => {
            tracing::error!("Health error: {}", e);
            Response::builder().status(500).body("Health failed".into()).unwrap()
        }
    }
}

async fn get_outcomes(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let result = sqlx::query!(
        r#"
        SELECT status as "status!", COUNT(*) as "count!"
        FROM crawl_events
        WHERE timestamp > NOW() - interval '24 hours'
        GROUP BY status
        "#
    )
    .fetch_all(&state.pool)
    .await;

    match result {
        Ok(rows) => {
            let metrics = rows.into_iter().map(|row| OutcomeMetric {
                status: row.status,
                count: row.count,
            }).collect::<Vec<_>>();
            Json(metrics).into_response()
        }
        Err(e) => {
            tracing::error!("Outcomes error: {}", e);
            Response::builder().status(500).body("Outcomes failed".into()).unwrap()
        }
    }
}

async fn replay_handler(
    State(state): State<Arc<AppState>>,
    Path((timestamp_str, url_str)): Path<(String, String)>,
) -> impl IntoResponse {
    let replay_url = match ReplayUrl::parse(&timestamp_str, &url_str) {
        Ok(u) => u,
        Err(_) => return Response::builder().status(400).body("Invalid timestamp or URL".into()).unwrap(),
    };

    let snapshot = match state.resolver.resolve(&replay_url.original_url, replay_url.timestamp).await {
        Ok(Some(s)) => s,
        Ok(None) => return Response::builder().status(404).body("Snapshot not found".into()).unwrap(),
        Err(e) => {
            tracing::error!("DB Error: {}", e);
            return Response::builder().status(500).body("Internal Server Error".into()).unwrap();
        }
    };

    let raw_data = match state.warc_reader.read_record(&snapshot.warc_file, snapshot.offset, snapshot.length).await {
        Ok(d) => d,
        Err(e) => {
            tracing::error!("Storage Error: {}", e);
            return Response::builder().status(500).body("Error reading archive".into()).unwrap();
        }
    };

    // Replay Logic: Rewrite if HTML
    if snapshot.content_type.contains("html") {
        let rewriter = Rewriter::new(snapshot.timestamp, snapshot.url);
        let rewritten_body = rewriter.rewrite_html(&raw_data);
        
        Response::builder()
            .status(snapshot.status_code)
            .header("Content-Type", "text/html")
            .body(rewritten_body.into())
            .unwrap()
    } else {
        Response::builder()
            .status(snapshot.status_code)
            .header("Content-Type", snapshot.content_type)
            .body(raw_data.into())
            .unwrap()
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
            Response::builder().status(500).body("Search failed".into()).unwrap()
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
        Ok(None) => Response::builder().status(404).body("No snapshot found for given URL and time".into()).unwrap(),
        Err(e) => {
            tracing::error!("Resolve error: {}", e);
            Response::builder().status(500).body("Internal error".into()).unwrap()
        }
    }
}

async fn get_snapshots_v1(
    State(state): State<Arc<AppState>>,
    Query(params): Query<SnapshotsQuery>,
) -> impl IntoResponse {
    let limit = params.limit.unwrap_or(50);
    
    // Simple query: in v2 we would add date range filters
    let result = sqlx::query_as!(
        Snapshot,
        r#"
        SELECT id, url, timestamp, warc_file, offset, length, sha256, status_code as "status_code!", content_type, payload_hash
        FROM snapshots
        WHERE url = $1
        ORDER BY timestamp DESC
        LIMIT $2
        "#,
        params.url,
        limit
    )
    .fetch_all(&state.pool)
    .await;

    match result {
        Ok(snapshots) => Json(snapshots).into_response(),
        Err(e) => {
            tracing::error!("Snapshots error: {}", e);
            Response::builder().status(500).body("Error fetching snapshots".into()).unwrap()
        }
    }
}

async fn get_snapshot(Path(id): Path<Uuid>, State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let result = sqlx::query_as!(
        Snapshot,
        r#"
        SELECT id, url, timestamp, warc_file, offset, length, sha256, status_code as "status_code!", content_type, payload_hash
        FROM snapshots
        WHERE id = $1
        "#,
        id
    )
    .fetch_optional(&state.pool)
    .await;

    match result {
        Ok(Some(s)) => Json(s).into_response(),
        Ok(None) => Response::builder().status(404).body("Snapshot not found").unwrap().into_response(),
        Err(e) => {
            tracing::error!("Snapshot error: {}", e);
            Response::builder().status(500).body("Error fetching snapshot").unwrap().into_response()
        }
    }
}

async fn start_crawl(State(state): State<Arc<AppState>>, Json(payload): Json<serde_json::Value>) -> Json<serde_json::Value> {
    // Basic crawl trigger (placeholder logic)
    let url = payload.get("url").and_then(|v| v.as_str()).unwrap_or("");
    tracing::info!("Starting crawl for: {}", url);
    Json(serde_json::json!({"status": "queued", "url": url}))
}
