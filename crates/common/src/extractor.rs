use scraper::{Html, Selector};
use std::sync::Arc;

pub struct ExtractionResult {
    pub title: String,
    pub text_content: String,
    pub metadata: std::collections::HashMap<String, String>,
}

pub trait ExtractorPlugin: Send + Sync {
    fn name(&self) -> &str;
    fn extract(&self, html: &Html) -> ExtractionResult;
    fn can_handle(&self, url: &str) -> bool {
        true
    }
}

pub struct DefaultExtractor;

impl ExtractorPlugin for DefaultExtractor {
    fn name(&self) -> &str {
        "default"
    }

    fn extract(&self, document: &Html) -> ExtractionResult {
        let title_selector = Selector::parse("title").unwrap();
        let title = document
            .select(&title_selector)
            .next()
            .map(|el| el.text().collect::<String>())
            .unwrap_or_else(|| "Untitled".to_string());

        // Extract text from body, skipping script and style tags
        let body_selector = Selector::parse("body").unwrap();
        let mut text_content = String::new();

        if let Some(body) = document.select(&body_selector).next() {
            for node in body.descendants() {
                if let Some(text) = node.value().as_text() {
                    let is_boilerplate = node
                        .parent()
                        .map(|p| {
                            if let Some(el) = p.value().as_element() {
                                let name = el.name();
                                name == "script"
                                    || name == "style"
                                    || name == "nav"
                                    || name == "footer"
                                    || name == "aside"
                            } else {
                                false
                            }
                        })
                        .unwrap_or(false);

                    if !is_boilerplate {
                        text_content.push_str(text);
                        text_content.push(' ');
                    }
                }
            }
        }

        ExtractionResult {
            title,
            text_content: text_content.trim().to_string(),
            metadata: std::collections::HashMap::new(),
        }
    }
}

pub struct PluginRegistry {
    plugins: Vec<Arc<dyn ExtractorPlugin>>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            plugins: vec![Arc::new(DefaultExtractor)],
        }
    }

    pub fn register(&mut self, plugin: Arc<dyn ExtractorPlugin>) {
        self.plugins.push(plugin);
    }

    pub fn extract_all(&self, url: &str, html_content: &str) -> Vec<ExtractionResult> {
        let document = Html::parse_document(html_content);
        self.plugins
            .iter()
            .filter(|p| p.can_handle(url))
            .map(|p| p.extract(&document))
            .collect()
    }
}

pub fn extract_text(html_content: &str) -> ExtractionResult {
    let registry = PluginRegistry::new();
    let results = registry.extract_all("", html_content);
    results.into_iter().next().unwrap()
}
