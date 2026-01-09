use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use archive_semantic::{NotificationChannel, ClassificationResult};
use tracing::{info, error};

#[async_trait]
pub trait NotificationDispatcher: Send + Sync {
    async fn dispatch(&self, url: &str, result: &ClassificationResult, channel: &NotificationChannel) -> anyhow::Result<()>;
}

pub struct MultiChannelDispatcher {
    client: reqwest::Client,
}

impl MultiChannelDispatcher {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    async fn send_webhook(&self, endpoint: &str, payload: serde_json::Value) -> anyhow::Result<()> {
        let resp = self.client.post(endpoint)
            .json(&payload)
            .send()
            .await?;
        
        if resp.status().is_success() {
            Ok(())
        } else {
            Err(anyhow::anyhow!("Webhook failed with status: {}", resp.status()))
        }
    }

    async fn send_slack(&self, webhook_url: &str, url: &str, result: &ClassificationResult) -> anyhow::Result<()> {
        let payload = serde_json::json!({
            "text": format!("🔔 *ArchiveStream Alert* for <{}|{}>", url, url),
            "blocks": [
                {
                    "type": "section",
                    "text": {
                        "type": "mrkdwn",
                        "text": format!("🔔 *ArchiveStream Alert*\n*URL:* <{}|{}>\n*Summary:* {}", url, url, result.summary)
                    }
                },
                {
                    "type": "section",
                    "fields": [
                        {
                            "type": "mrkdwn",
                            "text": format!("*Categories:*\n{}", result.categories.iter().map(|c| format!("{:?}", c)).collect::<Vec<_>>().join(", "))
                        },
                        {
                            "type": "mrkdwn",
                            "text": format!("*Confidence:*\n{:.2}%", result.confidence * 100.0)
                        }
                    ]
                }
            ]
        });

        self.send_webhook(webhook_url, payload).await
    }
}

#[async_trait]
impl NotificationDispatcher for MultiChannelDispatcher {
    async fn dispatch(&self, url: &str, result: &ClassificationResult, channel: &NotificationChannel) -> anyhow::Result<()> {
        match channel {
            NotificationChannel::Webhook(endpoint) => {
                info!("Dispatching Webhook to {}", endpoint);
                let payload = serde_json::json!({
                    "url": url,
                    "summary": result.summary,
                    "categories": result.categories,
                    "confidence": result.confidence,
                    "timestamp": chrono::Utc::now(),
                });
                self.send_webhook(endpoint, payload).await
            }
            NotificationChannel::Slack { webhook_url, .. } => {
                info!("Dispatching Slack notification");
                self.send_slack(webhook_url, url, result).await
            }
            NotificationChannel::Email(recipient) => {
                info!("Dispatching Email to {} (MOCK)", recipient);
                // In a real implementation, use lettre or an API like SendGrid
                Ok(())
            }
        }
    }
}
