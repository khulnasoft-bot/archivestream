use opensearch::{OpenSearch, SearchParts};
use serde_json::{json, Value};
use anyhow::Result;

pub struct SearchService {
    client: OpenSearch,
}

impl SearchService {
    pub fn new(client: OpenSearch) -> Self {
        Self { client }
    }

    pub async fn search(&self, query: &str) -> Result<Vec<Value>> {
        let response = self.client
            .search(SearchParts::Index(&["snapshots"]))
            .body(json!({
                "query": {
                    "multi_match": {
                        "query": query,
                        "fields": ["title^2", "content", "url"]
                    }
                },
                "highlight": {
                    "fields": {
                        "content": {}
                    }
                }
            }))
            .send()
            .await?;

        let response_body = response.json::<Value>().await?;
        let hits = response_body["hits"]["hits"].as_array().unwrap_or(&vec![]).clone();
        
        let results = hits.into_iter().map(|hit| {
            let source = hit["_source"].clone();
            let highlight = hit["highlight"]["content"].as_array()
                .and_then(|a| a.first())
                .cloned()
                .unwrap_or(Value::String("...".into()));
            
            json!({
                "snapshot_id": source["snapshot_id"],
                "url": source["url"],
                "title": source["title"],
                "timestamp": source["timestamp"],
                "snippet": highlight
            })
        }).collect();

        Ok(results)
    }
}
