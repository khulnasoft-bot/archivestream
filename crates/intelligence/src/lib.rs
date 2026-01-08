use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub summary: Option<String>,
    pub categories: Vec<ScoredCategory>,
    pub sentiment: Option<f32>, // -1.0 to 1.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoredCategory {
    pub name: String,
    pub confidence: f32, // 0.0 to 1.0
}

#[async_trait]
pub trait IntelligenceEngine: Send + Sync {
    /// Analyze text content to extract categories and sentiment
    async fn analyze(&self, text: &str) -> anyhow::Result<AnalysisResult>;

    /// Generate a vector embedding for the given text (for semantic search)
    async fn embed(&self, text: &str) -> anyhow::Result<Vec<f32>>;
    
    /// Summarize the difference between two text blocks
    async fn summarize_diff(&self, old_text: &str, new_text: &str) -> anyhow::Result<String>;
}

/// A pass-through engine that uses our existing regex-based logic
/// but wraps it in the new Intelligence interface.
pub struct RuleBasedEngine;

#[async_trait]
impl IntelligenceEngine for RuleBasedEngine {
    async fn analyze(&self, _text: &str) -> anyhow::Result<AnalysisResult> {
        // Placeholder: connect to archive-semantic's classifier here
        Ok(AnalysisResult {
            summary: None,
            categories: vec![],
            sentiment: None,
        })
    }

    async fn embed(&self, _text: &str) -> anyhow::Result<Vec<f32>> {
        // Rule-based engine cannot produce embeddings
        Ok(vec![]) 
    }

    async fn summarize_diff(&self, _old_text: &str, _new_text: &str) -> anyhow::Result<String> {
        Ok("Content changed.".to_string())
    }
}
