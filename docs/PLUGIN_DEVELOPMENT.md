# ArchiveStream Plugin Development Guide

## Overview

ArchiveStream supports custom plugins for:
- **Extractors**: Custom content extraction logic
- **Classifiers**: Domain-specific semantic classification
- **Notification Channels**: Custom alert delivery methods

## Creating an Extractor Plugin

Extractors allow you to define custom logic for parsing specific types of web pages.

### Example: E-commerce Price Extractor

```rust
use archive_common::extractor::{ExtractorPlugin, ExtractionResult};
use scraper::{Html, Selector};
use std::collections::HashMap;

pub struct EcommerceExtractor;

impl ExtractorPlugin for EcommerceExtractor {
    fn name(&self) -> &str {
        "ecommerce"
    }

    fn can_handle(&self, url: &str) -> bool {
        url.contains("amazon.com") 
            || url.contains("ebay.com")
            || url.contains("shopify.com")
    }

    fn extract(&self, html: &Html) -> ExtractionResult {
        let mut metadata = HashMap::new();

        // Extract price
        if let Some(price) = self.extract_price(html) {
            metadata.insert("price".to_string(), price);
        }

        // Extract product name
        let title = self.extract_title(html);

        ExtractionResult {
            title,
            text_content: String::new(),
            metadata,
        }
    }
}

impl EcommerceExtractor {
    fn extract_price(&self, html: &Html) -> Option<String> {
        let selectors = vec![
            ".price",
            "[itemprop='price']",
            ".product-price",
        ];

        for sel_str in selectors {
            if let Ok(selector) = Selector::parse(sel_str) {
                if let Some(elem) = html.select(&selector).next() {
                    return Some(elem.text().collect::<String>().trim().to_string());
                }
            }
        }
        None
    }

    fn extract_title(&self, html: &Html) -> String {
        let selector = Selector::parse("h1").unwrap();
        html.select(&selector)
            .next()
            .map(|el| el.text().collect::<String>())
            .unwrap_or_else(|| "Untitled".to_string())
    }
}
```

### Registering Your Plugin

```rust
use archive_common::extractor::PluginRegistry;
use std::sync::Arc;

let mut registry = PluginRegistry::new();
registry.register(Arc::new(EcommerceExtractor));
```

## Creating a Custom Classifier

```rust
use archive_intelligence::{IntelligenceEngine, AnalysisResult, ScoredCategory};
use async_trait::async_trait;

pub struct CustomClassifier;

#[async_trait]
impl IntelligenceEngine for CustomClassifier {
    async fn analyze(&self, added: &str, removed: &str) -> Result<AnalysisResult, String> {
        // Your custom classification logic
        let categories = vec![
            ScoredCategory {
                category: "custom_event".to_string(),
                confidence: 0.95,
            }
        ];

        Ok(AnalysisResult {
            categories,
            summary: Some("Custom analysis complete".to_string()),
            embeddings: None,
        })
    }
}
```

## Creating a Notification Channel

```rust
use archive_notification::{NotificationChannel, NotificationPayload};
use async_trait::async_trait;

pub struct DiscordChannel {
    webhook_url: String,
}

#[async_trait]
impl NotificationChannel for DiscordChannel {
    fn name(&self) -> &str {
        "discord"
    }

    async fn send(&self, payload: &NotificationPayload) -> Result<(), String> {
        let client = reqwest::Client::new();
        let body = serde_json::json!({
            "content": format!("🔔 {} changed!", payload.url),
            "embeds": [{
                "title": "View Diff",
                "url": payload.diff_url,
                "description": &payload.summary,
            }]
        });

        client.post(&self.webhook_url)
            .json(&body)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        Ok(())
    }
}
```

## Publishing Your Plugin

1. Create a new crate: `cargo new archivestream-plugin-yourname`
2. Add dependencies:
   ```toml
   [dependencies]
   archive-common = "0.1"
   archive-intelligence = "0.1"
   ```
3. Publish to crates.io: `cargo publish`
4. Share on [ArchiveStream Community Plugins](https://github.com/archivestream/plugins)

## Community Plugins

Browse existing plugins at: https://github.com/archivestream/plugins

- `archivestream-plugin-reddit` - Reddit-specific content extraction
- `archivestream-plugin-twitter` - Twitter/X thread archiving
- `archivestream-plugin-github` - GitHub repository snapshots
