use serde::Serialize;
use regex::Regex;

#[derive(Debug, Serialize, Clone, PartialEq)]
pub enum SemanticCategory {
    PrivacyPolicy,
    PriceChange,
    BreakingNews,
    ContentUpdate,
    StructuralChange,
    Unknown,
}

#[derive(Debug, Serialize)]
pub struct ClassificationResult {
    pub categories: Vec<SemanticCategory>,
    pub summary: String,
    pub confidence: f32,
}

pub struct Classifier {
    privacy_regex: Regex,
    price_regex: Regex,
    news_regex: Regex,
}

impl Classifier {
    pub fn new() -> Self {
        Self {
            privacy_regex: Regex::new(r"(?i)(privacy|policy|terms|cookie|consent)").unwrap(),
            price_regex: Regex::new(r"(?i)(\$|price|cost|usd|eur|gbp)").unwrap(),
            news_regex: Regex::new(r"(?i)(breaking|announced|acquisition|official|breaking next)").unwrap(),
        }
    }

    pub fn classify(&self, added_text: &str, removed_text: &str) -> ClassificationResult {
        let mut categories = Vec::new();
        let mut summary_parts = Vec::new();

        // 1. Check for Privacy/Legal
        if self.privacy_regex.is_match(added_text) || self.privacy_regex.is_match(removed_text) {
            categories.push(SemanticCategory::PrivacyPolicy);
            summary_parts.push("Detected legal or privacy policy updates.");
        }

        // 2. Check for Price Changes
        if self.price_regex.is_match(added_text) && self.price_regex.is_match(removed_text) {
            categories.push(SemanticCategory::PriceChange);
            summary_parts.push("Possible product price adjustments detected.");
        }

        // 3. Check for Breaking News
        if self.news_regex.is_match(added_text) {
            categories.push(SemanticCategory::BreakingNews);
            summary_parts.push("Significant news or announcement detected.");
        }

        // 4. Default to Content Update if something changed but no specific category matched
        if categories.is_empty() && (!added_text.is_empty() || !removed_text.is_empty()) {
            categories.push(SemanticCategory::ContentUpdate);
            summary_parts.push("General content update detected.");
        }

        if categories.is_empty() {
            categories.push(SemanticCategory::Unknown);
            summary_parts.push("No significant semantic changes identified.");
        }

        ClassificationResult {
            categories,
            summary: summary_parts.join(" "),
            confidence: 0.8, // Rule-based is fairly confident for these simple rules
        }
    }
}

impl Default for Classifier {
    fn default() -> Self {
        Self::new()
    }
}
