use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use crate::models::Rule;

pub async fn init_pool(database_url: &str) -> Result<SqlitePool, sqlx::Error> {
    // Create database file if it doesn't exist
    if database_url.starts_with("sqlite:") {
        let path = database_url.trim_start_matches("sqlite:");
        let path = path.trim_start_matches("./");
        if !std::path::Path::new(path).exists() {
            std::fs::File::create(path).ok();
        }
    }

    SqlitePoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await
}

pub async fn run_migrations(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS rules (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            category TEXT NOT NULL,
            subcategory TEXT,
            content TEXT NOT NULL,
            source TEXT NOT NULL,
            page INTEGER,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE VIRTUAL TABLE IF NOT EXISTS rules_fts USING fts5(
            title,
            content,
            category,
            content='rules',
            content_rowid='rowid'
        );

        CREATE TRIGGER IF NOT EXISTS rules_ai AFTER INSERT ON rules BEGIN
            INSERT INTO rules_fts(rowid, title, content, category)
            VALUES (NEW.rowid, NEW.title, NEW.content, NEW.category);
        END;

        CREATE TRIGGER IF NOT EXISTS rules_ad AFTER DELETE ON rules BEGIN
            INSERT INTO rules_fts(rules_fts, rowid, title, content, category)
            VALUES ('delete', OLD.rowid, OLD.title, OLD.content, OLD.category);
        END;

        CREATE TRIGGER IF NOT EXISTS rules_au AFTER UPDATE ON rules BEGIN
            INSERT INTO rules_fts(rules_fts, rowid, title, content, category)
            VALUES ('delete', OLD.rowid, OLD.title, OLD.content, OLD.category);
            INSERT INTO rules_fts(rowid, title, content, category)
            VALUES (NEW.rowid, NEW.title, NEW.content, NEW.category);
        END;
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_all_rules(pool: &SqlitePool) -> Result<Vec<Rule>, sqlx::Error> {
    sqlx::query_as::<_, Rule>("SELECT * FROM rules ORDER BY category, title")
        .fetch_all(pool)
        .await
}

pub async fn get_rule_by_id(pool: &SqlitePool, id: &str) -> Result<Option<Rule>, sqlx::Error> {
    sqlx::query_as::<_, Rule>("SELECT * FROM rules WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn create_rule(pool: &SqlitePool, rule: &Rule) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO rules (id, title, category, subcategory, content, source, page, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&rule.id)
    .bind(&rule.title)
    .bind(&rule.category)
    .bind(&rule.subcategory)
    .bind(&rule.content)
    .bind(&rule.source)
    .bind(&rule.page)
    .bind(&rule.created_at)
    .bind(&rule.updated_at)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn search_rules_fts(pool: &SqlitePool, query: &str) -> Result<Vec<Rule>, sqlx::Error> {
    sqlx::query_as::<_, Rule>(
        r#"
        SELECT r.* FROM rules r
        JOIN rules_fts fts ON r.rowid = fts.rowid
        WHERE rules_fts MATCH ?
        ORDER BY rank
        LIMIT 20
        "#,
    )
    .bind(query)
    .fetch_all(pool)
    .await
}
