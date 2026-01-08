use scraper::{Html, Selector};

pub struct ExtractionResult {
    pub title: String,
    pub text_content: String,
}

pub fn extract_text(html_content: &str) -> ExtractionResult {
    let document = Html::parse_document(html_content);
    
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
                // Check if parent is script or style
                let is_boilerplate = node.parent().map(|p| {
                    if let Some(el) = p.value().as_element() {
                        let name = el.name();
                        name == "script" || name == "style" || name == "nav" || name == "footer" || name == "aside"
                    } else {
                        false
                    }
                }).unwrap_or(false);

                if !is_boilerplate {
                    text_content.push_str(&text);
                    text_content.push(' ');
                }
            }
        }
    }

    ExtractionResult {
        title,
        text_content: text_content.trim().to_string(),
    }
}
