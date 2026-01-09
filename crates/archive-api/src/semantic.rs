use axum::{
    extract::{Query, State},
    response::IntoResponse,
    Json,
};
use std::sync::Arc;
use crate::{AppState, DiffQuery};
use crate::diff::DiffService;
use archive_semantic::Classifier;
use archive_common::replay::ReplayUrl;
use axum::http::StatusCode;

pub async fn get_semantic_change(
    State(state): State<Arc<AppState>>,
    Query(params): Query<DiffQuery>,
) -> impl IntoResponse {
    // 1. Resolve snapshots (reuse logic from diff)
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
        return (StatusCode::NOT_FOUND, "Snapshots not found").into_response();
    }

    let s1 = s1.unwrap();
    let s2 = s2.unwrap();

    let d1 = state.warc_reader.read_record(&s1.warc_file, s1.offset, s1.length).await.unwrap_or_default();
    let d2 = state.warc_reader.read_record(&s2.warc_file, s2.offset, s2.length).await.unwrap_or_default();

    let h1 = String::from_utf8_lossy(&d1);
    let h2 = String::from_utf8_lossy(&d2);

    // 2. Perform Diff
    let diff = DiffService::compute_diff(&h1, &h2, &params.from, &params.to);

    // 3. Extract text for classification (using just added/removed lines)
    let mut added_text = String::new();
    let mut removed_text = String::new();

    for change in diff.changes {
        match change.tag.as_str() {
            "added" => {
                added_text.push_str(&change.value);
                added_text.push(' ');
            }
            "removed" => {
                removed_text.push_str(&change.value);
                removed_text.push(' ');
            }
            _ => {}
        }
    }

    // 4. Classify & Summarize using Intelligence Engine (Phase 7)
    let text_to_analyze = format!("Added:\n{}\n\nRemoved:\n{}", added_text, removed_text);
    let analysis = state.intelligence_engine.analyze(&text_to_analyze).await.unwrap_or_else(|e| {
        tracing::error!("Intelligence analysis failed: {}", e);
        archive_intelligence::AnalysisResult {
            summary: Some("Error during intelligence analysis.".to_string()),
            categories: vec![],
            sentiment: None,
        }
    });

    // Also get a smart summary of changes if it's an LLM
    let smart_summary = state.intelligence_engine.summarize_diff(&h1, &h2).await.ok();

    // 5. Evaluate Alerts (Phase 8.1)
    let rules = sqlx::query_as::<_, archive_semantic::AlertRule>(
        "SELECT id, name, url_pattern, categories, channels, min_confidence, active FROM alert_rules WHERE active = true"
    )
    .fetch_all(&state.pool)
    .await.unwrap_or_default();

    let alert_engine = archive_semantic::AlertEngine::new(rules);
    
    // We need to convert Intelligence categories back to SemanticCategory for the alert engine
    let semantic_result = archive_semantic::ClassificationResult {
        categories: analysis.categories.iter().map(|c| match c.name.as_str() {
            "PrivacyPolicy" => archive_semantic::SemanticCategory::PrivacyPolicy,
            "PriceChange" => archive_semantic::SemanticCategory::PriceChange,
            "BreakingNews" => archive_semantic::SemanticCategory::BreakingNews,
            _ => archive_semantic::SemanticCategory::ContentUpdate,
        }).collect(),
        summary: analysis.summary.clone().unwrap_or_default(),
        confidence: analysis.categories.iter().map(|c| c.confidence).sum::<f32>() / analysis.categories.len().max(1) as f32,
    };

    let triggered_alerts = alert_engine.evaluate(&params.url, &semantic_result);
    
    for alert in &triggered_alerts {
        tracing::info!("ALERT TRIGGERED: {} for URL {}", alert.name, params.url);
        
        for channel in &alert.channels {
            let dispatcher = state.notification_dispatcher.clone();
            let url = params.url.clone();
            let result = semantic_result.clone();
            let channel = channel.clone();
            let pool = state.pool.clone();
            let rule_id = alert.id;

            tokio::spawn(async move {
                match dispatcher.dispatch(&url, &result, &channel).await {
                    Ok(_) => {
                        let _ = sqlx::query(
                            "INSERT INTO notification_history (id, rule_id, url, channel_type, status) VALUES ($1, $2, $3, $4, $5)"
                        )
                        .bind(uuid::Uuid::new_v4())
                        .bind(rule_id)
                        .bind(&url)
                        .bind(format!("{:?}", channel))
                        .bind("success")
                        .execute(&pool)
                        .await;
                    }
                    Err(e) => {
                        tracing::error!("Failed to dispatch notification: {}", e);
                        let _ = sqlx::query(
                            "INSERT INTO notification_history (id, rule_id, url, channel_type, status) VALUES ($1, $2, $3, $4, $5)"
                        )
                        .bind(uuid::Uuid::new_v4())
                        .bind(rule_id)
                        .bind(&url)
                        .bind(format!("{:?}", channel))
                        .bind("failed")
                        .execute(&pool)
                        .await;
                    }
                }
            });
        }
    }

    Json(serde_json::json!({
        "from": params.from,
        "to": params.to,
        "url": params.url,
        "analysis": analysis,
        "smart_summary": smart_summary,
        "stats": diff.summary,
        "alerts_triggered": triggered_alerts.len()
    })).into_response()
}


