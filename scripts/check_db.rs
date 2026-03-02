use sqlx::sqlite::SqlitePoolOptions;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:./rulecraft.db".to_string());
    
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect(&db_url)
        .await?;

    // Check total count
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM rules").fetch_one(&pool).await?;
    println!("Total rules: {}", count.0);

    // Check Spells category
    let spells: Vec<(String, String)> = sqlx::query_as("SELECT id, title FROM rules WHERE category = 'Spells'")
        .fetch_all(&pool).await?;
    println!("\nSpells category ({} rules):", spells.len());
    for (id, title) in &spells {
        println!("  [{}] {}", id, title);
    }

    // Fuzzy test
    let fuzzy: Vec<(String, String, String)> = sqlx::query_as(
        "SELECT id, title, category FROM rules WHERE LOWER(title) LIKE '%fireball%' OR LOWER(content) LIKE '%fireball%' LIMIT 5"
    ).fetch_all(&pool).await?;
    println!("\nDirect fuzzy match for 'fireball' ({} results):", fuzzy.len());
    for (id, title, cat) in &fuzzy {
        println!("  [{}/{}] {}", cat, id, title);
    }

    Ok(())
}
