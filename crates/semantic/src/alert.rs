use serde::{Deserialize, Serialize};
use crate::classifier::{SemanticCategory, ClassificationResult};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    pub id: Uuid,
    pub name: String,
    pub url_pattern: String, // Regex or Glob
    pub categories: Vec<SemanticCategory>,
    pub channels: Vec<NotificationChannel>,
    pub min_confidence: f32,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "config")]
pub enum NotificationChannel {
    Email(String),
    Webhook(String),
    Slack { channel: String, webhook_url: String },
}

pub struct AlertEngine {
    rules: Vec<AlertRule>,
}

impl AlertEngine {
    pub fn new(rules: Vec<AlertRule>) -> Self {
        Self { rules }
    }

    pub fn evaluate(&self, url: &str, result: &ClassificationResult) -> Vec<&AlertRule> {
        let mut triggered = Vec::new();

        for rule in &self.rules {
            if !rule.active { continue; }
            
            // 1. Check URL Pattern (simplified glob-to-regex or exact for now)
            if !url.contains(&rule.url_pattern) && rule.url_pattern != "*" {
                continue;
            }

            // 2. Check Confidence
            if result.confidence < rule.min_confidence {
                continue;
            }

            // 3. Check Categories
            for cat in &result.categories {
                if rule.categories.contains(cat) {
                    triggered.push(rule);
                    break;
                }
            }
        }

        triggered
    }
}
