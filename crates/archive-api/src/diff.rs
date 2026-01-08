use similar::{TextDiff, ChangeTag};
use serde::Serialize;
use anyhow::Result;
use archive_common::extractor::extract_text;

#[derive(Serialize)]
pub struct DiffResult {
    pub from_timestamp: String,
    pub to_timestamp: String,
    pub summary: DiffSummary,
    pub changes: Vec<DiffChange>,
}

#[derive(Serialize)]
pub struct DiffSummary {
    pub added: usize,
    pub removed: usize,
    pub unchanged: usize,
}

#[derive(Serialize)]
pub struct DiffChange {
    pub tag: String, // "added", "removed", "equal"
    pub value: String,
}

pub struct DiffService;

impl DiffService {
    pub fn compute_diff(from_html: &str, to_html: &str, from_ts: &str, to_ts: &str) -> DiffResult {
        let from_text = extract_text(from_html).text_content;
        let to_text = extract_text(to_html).text_content;

        let diff = TextDiff::from_lines(&from_text, &to_text);
        
        let mut added = 0;
        let mut removed = 0;
        let mut unchanged = 0;
        let mut changes = Vec::new();

        for change in diff.iter_all_changes() {
            let tag = match change.tag() {
                ChangeTag::Delete => {
                    removed += 1;
                    "removed".to_string()
                },
                ChangeTag::Insert => {
                    added += 1;
                    "added".to_string()
                },
                ChangeTag::Equal => {
                    unchanged += 1;
                    "equal".to_string()
                },
            };
            changes.push(DiffChange {
                tag,
                value: change.value().to_string(),
            });
        }

        DiffResult {
            from_timestamp: from_ts.to_string(),
            to_timestamp: to_ts.to_string(),
            summary: DiffSummary { added, removed, unchanged },
            changes,
        }
    }
}
