use async_trait::async_trait;
use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlPrediction {
    pub url: String,
    pub next_fetch_at: DateTime<Utc>,
    pub recommended_priority: i32,
    pub confidence: f32,
    pub change_probability: f32,
}

#[async_trait]
pub trait PredictiveEngine: Send + Sync {
    /// Predict the best time to recrawl a URL based on its change history
    async fn predict_next_crawl(&self, history: &[SnapshotHistory]) -> anyhow::Result<CrawlPrediction>;
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SnapshotHistory {
    pub timestamp: DateTime<Utc>,
    pub content_hash: String,
}



/// A pass-through engine that uses our existing regex-based logic
/// but wraps it in the new Intelligence interface.
pub struct RuleBasedEngine;

#[async_trait]
impl IntelligenceEngine for RuleBasedEngine {
    async fn analyze(&self, text: &str) -> anyhow::Result<AnalysisResult> {
        let classifier = archive_semantic::Classifier::new();
        // The rule-based classifier expects added/removed text. 
        // For a single block, we treat it as all added.
        let result = classifier.classify(text, "");
        
        Ok(AnalysisResult {
            summary: Some(result.summary),
            categories: result.categories.into_iter().map(|c| ScoredCategory {
                name: format!("{:?}", c),
                confidence: result.confidence,
            }).collect(),
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

/// A premium engine that uses OpenAI-compatible APIs for deep analysis.
pub struct LLMIntelligenceEngine {
    api_key: String,
    model: String,
    endpoint: String,
}

impl LLMIntelligenceEngine {
    pub fn new(api_key: String, model: String, endpoint: Option<String>) -> Self {
        Self {
            api_key,
            model,
            endpoint: endpoint.unwrap_or_else(|| "https://api.openai.com/v1".to_string()),
        }
    }
}

#[async_trait]
impl IntelligenceEngine for LLMIntelligenceEngine {
    async fn analyze(&self, text: &str) -> anyhow::Result<AnalysisResult> {
        let client = reqwest::Client::new();
        let prompt = format!(
            "Analyze the following web page content and provide a summary, categories, and sentiment (-1.0 to 1.0). \
             Respond in JSON format with fields: {{'summary': string, 'categories': [{{'name': string, 'confidence': float}}], 'sentiment': float}}. \n\nContent: {}",
            text.chars().take(2000).collect::<String>()
        );

        let resp = client.post(format!("{}/chat/completions", self.endpoint))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&serde_json::json!({
                "model": self.model,
                "messages": [{"role": "user", "content": prompt}],
                "response_format": { "type": "json_object" }
            }))
            .send()
            .await?;

        let json: serde_json::Value = resp.json().await?;
        let content = json["choices"][0]["message"]["content"].as_str().ok_or_else(|| anyhow::anyhow!("Invalid response"))?;
        let result: AnalysisResult = serde_json::from_str(content)?;
        
        Ok(result)
    }

    async fn embed(&self, text: &str) -> anyhow::Result<Vec<f32>> {
        let client = reqwest::Client::new();
        let resp = client.post(format!("{}/embeddings", self.endpoint))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&serde_json::json!({
                "model": "text-embedding-3-small",
                "input": text
            }))
            .send()
            .await?;

        let json: serde_json::Value = resp.json().await?;
        let embedding = json["data"][0]["embedding"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("Invalid embedding response"))?
            .iter()
            .map(|v| v.as_f64().unwrap_or(0.0) as f32)
            .collect();
        
        Ok(embedding)
    }

    async fn summarize_diff(&self, old_text: &str, new_text: &str) -> anyhow::Result<String> {
        let client = reqwest::Client::new();
        let prompt = format!(
            "Compare these two versions of a web page and summarize what changed. \n\nOld: {}\n\nNew: {}",
            old_text.chars().take(1000).collect::<String>(),
            new_text.chars().take(1000).collect::<String>()
        );

        let resp = client.post(format!("{}/chat/completions", self.endpoint))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&serde_json::json!({
                "model": self.model,
                "messages": [{"role": "user", "content": prompt}]
            }))
            .send()
            .await?;

        let json: serde_json::Value = resp.json().await?;
        let summary = json["choices"][0]["message"]["content"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid response"))?
            .to_string();
            
        Ok(summary)
    }
}

/// Combines rule-based and ML classification for high accuracy (Phase 7.1)
pub struct HybridEngine {
    rule_based: RuleBasedEngine,
    llm: Option<LLMIntelligenceEngine>,
}

impl HybridEngine {
    pub fn new(llm: Option<LLMIntelligenceEngine>) -> Self {
        Self {
            rule_based: RuleBasedEngine,
            llm,
        }
    }
}

#[async_trait]
impl IntelligenceEngine for HybridEngine {
    async fn analyze(&self, text: &str) -> anyhow::Result<AnalysisResult> {
        let rb_result = self.rule_based.analyze(text).await?;
        
        if let Some(ref llm) = self.llm {
            // If LLM available, merge results
            match llm.analyze(text).await {
                Ok(llm_result) => {
                    let mut categories = rb_result.categories;
                    // Add LLM categories if they are not already found by rule-based (or just combine)
                    for cat in llm_result.categories {
                        if !categories.iter().any(|c| c.name == cat.name) {
                            categories.push(cat);
                        }
                    }
                    
                    Ok(AnalysisResult {
                        summary: llm_result.summary.or(rb_result.summary),
                        categories,
                        sentiment: llm_result.sentiment,
                    })
                }
                Err(e) => {
                    tracing::warn!("LLM analysis failed, falling back to rule-based: {}", e);
                    Ok(rb_result)
                }
            }
        } else {
            Ok(rb_result)
        }
    }

    async fn embed(&self, text: &str) -> anyhow::Result<Vec<f32>> {
        if let Some(ref llm) = self.llm {
            llm.embed(text).await
        } else {
            self.rule_based.embed(text).await
        }
    }

    async fn summarize_diff(&self, old_text: &str, new_text: &str) -> anyhow::Result<String> {
        if let Some(ref llm) = self.llm {
            llm.summarize_diff(old_text, new_text).await
        } else {
            self.rule_based.summarize_diff(old_text, new_text).await
        }
    }
}

/// A predictor that uses historical change data to schedule next crawls (Phase 7.3)
pub struct StandardPredictor;

#[async_trait]
impl PredictiveEngine for StandardPredictor {
    async fn predict_next_crawl(&self, history: &[SnapshotHistory]) -> anyhow::Result<CrawlPrediction> {
        if history.len() < 2 {
            // Not enough data, default to 24 hours
            return Ok(CrawlPrediction {
                url: "".to_string(), // Caller should fill this
                next_fetch_at: Utc::now() + chrono::Duration::hours(24),
                recommended_priority: 5,
                confidence: 0.2,
                change_probability: 0.5,
            });
        }

        // Calculate average interval between changes
        let mut changes = 0;
        let mut total_interval = chrono::Duration::zero();
        let mut last_content_hash = &history[0].content_hash;
        let mut last_timestamp = history[0].timestamp;

        for entry in history.iter().skip(1) {
            if entry.content_hash != *last_content_hash {
                changes += 1;
                total_interval = total_interval + (entry.timestamp - last_timestamp);
                last_content_hash = &entry.content_hash;
            }
            last_timestamp = entry.timestamp;
        }

        if changes == 0 {
            // Stable content, double the interval or default to 1 week
            let last_fetch = history.last().unwrap().timestamp;
            let current_age = Utc::now() - last_fetch;
            let next_interval = (current_age * 2).max(chrono::Duration::days(7));
            
            return Ok(CrawlPrediction {
                url: "".to_string(),
                next_fetch_at: Utc::now() + next_interval,
                recommended_priority: 1,
                confidence: 0.6,
                change_probability: 0.1,
            });
        }

        let avg_change_interval = total_interval / (changes as i32);
        
        // Schedule next crawl at 80% of average interval to catch changes early
        let next_fetch_delta = (avg_change_interval.num_seconds() as f32 * 0.8) as i64;
        let next_fetch_at = Utc::now() + chrono::Duration::seconds(next_fetch_delta);

        Ok(CrawlPrediction {
            url: "".to_string(),
            next_fetch_at,
            recommended_priority: (changes * 2).min(10) as i32,
            confidence: (changes as f32 / history.len() as f32).min(0.9),
            change_probability: 0.8,
        })
    }
}


