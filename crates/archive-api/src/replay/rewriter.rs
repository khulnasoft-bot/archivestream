use lol_html::{element, HtmlRewriter, Settings};
use archive_common::replay::ReplayUrl;
use chrono::{DateTime, Utc};
use url::Url;

pub struct Rewriter {
    timestamp: DateTime<Utc>,
    base_url: String,
}

impl Rewriter {
    pub fn new(timestamp: DateTime<Utc>, base_url: String) -> Self {
        Self { timestamp, base_url }
    }

    pub fn rewrite_html(&self, html_content: &[u8]) -> Vec<u8> {
        let mut output = Vec::new();
        
        let mut rewriter = HtmlRewriter::new(
            Settings {
                element_content_handlers: vec![
                    element!("a[href], img[src], script[src], link[href], form[action]", |el| {
                        let attr_name = if el.has_attribute("href") { "href" }
                            else if el.has_attribute("src") { "src" }
                            else { "action" };

                        if let Some(attr_val) = el.get_attribute(attr_name) {
                            let rewritten = self.rewrite_url(&attr_val);
                            el.set_attribute(attr_name, &rewritten).ok();
                        }
                        Ok(())
                    }),
                    element!("style", |el| {
                        // TODO: Implement CSS rewrite for inline styles
                        Ok(())
                    })
                ],
                ..Settings::default()
            },
            |c: &[u8]| output.extend_from_slice(c)
        );

        rewriter.write(html_content).unwrap();
        rewriter.end().unwrap();
        
        output
    }

    fn rewrite_url(&self, target_url: &str) -> String {
        // Skip data URIs, fragments, etc.
        if target_url.starts_with("data:") || target_url.starts_with('#') || target_url.starts_with("javascript:") {
            return target_url.to_string();
        }

        if let Ok(full_url) = self.resolve_absolute(target_url) {
            let replay = ReplayUrl {
                timestamp: self.timestamp,
                original_url: full_url,
            };
            return replay.format();
        }
        
        target_url.to_string()
    }

    fn resolve_absolute(&self, target: &str) -> Result<String, url::ParseError> {
        let base = Url::parse(&self.base_url)?;
        let resolved = base.join(target)?;
        Ok(resolved.to_string())
    }
}
