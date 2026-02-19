use serde::{Deserialize, Serialize};

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
}

fn chrono_now() -> String {
    // Simple ISO 8601 timestamp without chrono dependency
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", duration.as_secs())
}
