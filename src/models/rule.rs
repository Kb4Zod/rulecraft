use serde::{Deserialize, Serialize};
use pulldown_cmark::{Parser, Event};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Rule {
    pub id: String,
    pub title: String,
    pub category: String,
    pub subcategory: Option<String>,
    pub content: String,
    pub source: String,
    pub page: Option<i32>,
    pub created_at: String,
    pub updated_at: String,
}

impl Rule {
    pub fn new(title: String, category: String, content: String, source: String) -> Self {
        let now = chrono_now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            title,
            category,
            subcategory: None,
            content,
            source,
            page: None,
            created_at: now.clone(),
            updated_at: now,
        }
    }
    pub fn excerpt(&self, max_len: usize) -> String {
        let parser = Parser::new(&self.content);
        let mut text_elements = Vec::new();
        
        for event in parser {
            match event {
                Event::Text(text) => text_elements.push(text.into_string()),
                Event::Code(text) => text_elements.push(text.into_string()),
                _ => {} // Ignore tags like headers, strong, emphasis, lists
            }
        }
        
        // Join the text elements with spaces to form a continuous string
        let raw_text = text_elements.join(" ").replace("  ", " ");
        
        if raw_text.len() <= max_len {
            raw_text
        } else {
            // Find a good breaking point (space) or just hard-cut if none found near
            let mut end_idx = max_len;
            while end_idx > 0 && !raw_text.is_char_boundary(end_idx) {
                end_idx -= 1;
            }
            format!("{}...", &raw_text[..end_idx])
        }
    }
}

fn chrono_now() -> String {
    // Simple ISO 8601 timestamp without chrono dependency
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", duration.as_secs())
}
