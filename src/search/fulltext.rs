use sqlx::SqlitePool;
use crate::models::Rule;

/// Search rules using SQLite FTS5 full-text search
pub async fn search(pool: &SqlitePool, query: &str) -> Result<Vec<Rule>, sqlx::Error> {
    // Escape special FTS5 characters and prepare query
    let sanitized = sanitize_fts_query(query);

    if sanitized.is_empty() {
        return Ok(vec![]);
    }

    crate::db::search_rules_fts(pool, &sanitized).await
}

/// Sanitize query for FTS5
fn sanitize_fts_query(query: &str) -> String {
    // Remove special FTS5 operators for basic search
    query
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" OR ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_fts_query() {
        assert_eq!(sanitize_fts_query("attack action"), "attack OR action");
        assert_eq!(sanitize_fts_query("AC (armor class)"), "AC OR armor OR class");
    }
}
