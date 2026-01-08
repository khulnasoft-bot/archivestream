use axum::{
    extract::{Query, State},
    response::IntoResponse,
    Json,
};
use std::sync::Arc;
use crate::{AppState, DiffQuery};
use crate::diff::DiffService;
use archive_semantic::{Classifier, ClassificationResult};
use archive_common::replay::ReplayUrl;
use axum::response::Response;

pub async fn get_semantic_change(
    State(state): State<Arc<AppState>>,
    Query(params): Query<DiffQuery>,
) -> impl IntoResponse {
    // 1. Resolve snapshots (reuse logic from diff)
    let ts_from = match ReplayUrl::parse(&params.from, &params.url) {
        Ok(u) => u.timestamp,
        Err(_) => return Response::builder().status(400).body("Invalid FROM timestamp".into()).unwrap().into_response(),
    };
    let ts_to = match ReplayUrl::parse(&params.to, &params.url) {
        Ok(u) => u.timestamp,
        Err(_) => return Response::builder().status(400).body("Invalid TO timestamp".into()).unwrap().into_response(),
    };

    let s1 = state.resolver.resolve(&params.url, ts_from).await.ok().flatten();
    let s2 = state.resolver.resolve(&params.url, ts_to).await.ok().flatten();

    if s1.is_none() || s2.is_none() {
        return Response::builder().status(404).body("Snapshots not found".into()).unwrap().into_response();
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
        match change.change_type.as_str() {
            "insert" => {
                added_text.push_str(&change.text);
                added_text.push(' ');
            }
            "delete" => {
                removed_text.push_str(&change.text);
                removed_text.push(' ');
            }
            _ => {}
        }
    }

    // 4. Classify
    let classifier = Classifier::new();
    let classification = classifier.classify(&added_text, &removed_text);

    Json(serde_json::json!({
        "from": params.from,
        "to": params.to,
        "url": params.url,
        "classification": classification,
        "stats": diff.summary
    })).into_response()
}
