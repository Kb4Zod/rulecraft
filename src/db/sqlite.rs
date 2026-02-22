use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use crate::models::Rule;
use std::path::Path;

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
    // Create schema
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

    // Seed initial data if table is empty
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM rules")
        .fetch_one(pool)
        .await?;

    if count.0 == 0 {
        // Check if YAML rules directory exists with files
        let yaml_dir = Path::new("data/rules");
        let has_yaml_files = yaml_dir.exists()
            && yaml_dir.is_dir()
            && std::fs::read_dir(yaml_dir)
                .map(|mut entries| entries.any(|e| {
                    e.ok()
                        .map(|entry| entry.path().extension().map(|ext| ext == "yaml").unwrap_or(false))
                        .unwrap_or(false)
                }))
                .unwrap_or(false);

        if has_yaml_files {
            tracing::info!("YAML rules directory found at data/rules/ - run import_rules to load them");
        } else {
            // Fall back to inline seeds for Docker without volume mount
            seed_rules(pool).await?;
        }
    }

    Ok(())
}

async fn seed_rules(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query(r#"
        INSERT INTO rules (id, title, category, subcategory, content, source, page, created_at, updated_at)
        VALUES
            ('adv-disadv', 'Advantage and Disadvantage', 'Combat', 'Rolling',
             'Sometimes a special ability or spell tells you that you have advantage or disadvantage on an ability check, a saving throw, or an attack roll. When that happens, you roll a second d20 when you make the roll. Use the higher of the two rolls if you have advantage, and use the lower roll if you have disadvantage.',
             'Player''s Handbook 2024', 25, datetime('now'), datetime('now')),

            ('sneak-attack', 'Sneak Attack', 'Combat', 'Class Features',
             'Once per turn, you can deal extra 1d6 damage to one creature you hit with an attack if you have advantage on the attack roll. The attack must use a finesse or a ranged weapon. You don''t need advantage on the attack roll if another enemy of the target is within 5 feet of it, that enemy isn''t incapacitated, and you don''t have disadvantage on the attack roll.',
             'Player''s Handbook 2024', 98, datetime('now'), datetime('now')),

            ('opportunity-attack', 'Opportunity Attack', 'Combat', 'Actions',
             'You can make an opportunity attack when a hostile creature that you can see moves out of your reach. To make the opportunity attack, you use your reaction to make one melee attack against the provoking creature. The attack occurs right before the creature leaves your reach.',
             'Player''s Handbook 2024', 195, datetime('now'), datetime('now')),

            ('concentration', 'Concentration', 'Spellcasting', 'Mechanics',
             'Some spells require you to maintain concentration to keep their magic active. If you lose concentration, such a spell ends. Taking damage can break your concentration. When you take damage while concentrating on a spell, make a Constitution saving throw. The DC equals 10 or half the damage taken, whichever is higher.',
             'Player''s Handbook 2024', 233, datetime('now'), datetime('now')),

            ('cover', 'Cover', 'Combat', 'Environment',
             'Walls, trees, creatures, and other obstacles can provide cover during combat. Half cover grants +2 to AC and Dexterity saving throws. Three-quarters cover grants +5 to AC and Dexterity saving throws. Total cover means a target cannot be targeted directly by an attack or spell.',
             'Player''s Handbook 2024', 196, datetime('now'), datetime('now')),

            ('attack-action', 'Attack Action', 'Combat', 'Actions',
             'When you take the Attack action, you make one melee or ranged attack. Certain features, such as the Extra Attack feature of the Fighter, let you make more than one attack with this action.',
             'Player''s Handbook 2024', 189, datetime('now'), datetime('now')),

            ('dash-action', 'Dash Action', 'Combat', 'Actions',
             'When you take the Dash action, you gain extra movement for the current turn. The increase equals your speed, after applying any modifiers.',
             'Player''s Handbook 2024', 189, datetime('now'), datetime('now')),

            ('dodge-action', 'Dodge Action', 'Combat', 'Actions',
             'When you take the Dodge action, you focus entirely on avoiding attacks. Until the start of your next turn, any attack roll made against you has disadvantage if you can see the attacker, and you make Dexterity saving throws with advantage.',
             'Player''s Handbook 2024', 189, datetime('now'), datetime('now')),

            ('prone', 'Prone Condition', 'Conditions', NULL,
             'A prone creature''s only movement option is to crawl, unless it stands up. The creature has disadvantage on attack rolls. An attack roll against the creature has advantage if the attacker is within 5 feet. Otherwise, the attack roll has disadvantage.',
             'Player''s Handbook 2024', 369, datetime('now'), datetime('now')),

            ('grappled', 'Grappled Condition', 'Conditions', NULL,
             'A grappled creature''s speed becomes 0, and it can''t benefit from any bonus to its speed. The condition ends if the grappler is incapacitated or if an effect removes the grappled creature from the grappler''s reach.',
             'Player''s Handbook 2024', 367, datetime('now'), datetime('now'))
    "#)
    .execute(pool)
    .await?;

    tracing::info!("Seeded {} initial rules", 10);
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

/// Fuzzy search using LIKE patterns and prefix matching
/// Searches title, content, and category with case-insensitive matching
pub async fn fuzzy_search(pool: &SqlitePool, query: &str, limit: i32) -> Result<Vec<Rule>, sqlx::Error> {
    let query_lower = query.to_lowercase();
    let like_pattern = format!("%{}%", query_lower);
    let prefix_pattern = format!("{}%", query_lower);

    sqlx::query_as::<_, Rule>(
        r#"
        SELECT * FROM rules
        WHERE
            LOWER(title) LIKE ?1
            OR LOWER(content) LIKE ?1
            OR LOWER(category) LIKE ?1
        ORDER BY
            CASE
                WHEN LOWER(title) LIKE ?2 THEN 1
                WHEN LOWER(title) LIKE ?1 THEN 2
                WHEN LOWER(category) LIKE ?1 THEN 3
                ELSE 4
            END,
            title
        LIMIT ?3
        "#,
    )
    .bind(&like_pattern)
    .bind(&prefix_pattern)
    .bind(limit)
    .fetch_all(pool)
    .await
}

/// Upsert a rule - insert if new, update if exists
/// Returns true if a new rule was inserted, false if an existing rule was updated
pub async fn upsert_rule(pool: &SqlitePool, rule: &Rule) -> Result<bool, sqlx::Error> {
    // Check if rule exists
    let existing: Option<(String,)> = sqlx::query_as("SELECT id FROM rules WHERE id = ?")
        .bind(&rule.id)
        .fetch_optional(pool)
        .await?;

    if existing.is_some() {
        // Update existing rule
        sqlx::query(
            r#"
            UPDATE rules SET
                title = ?,
                category = ?,
                subcategory = ?,
                content = ?,
                source = ?,
                page = ?,
                updated_at = datetime('now')
            WHERE id = ?
            "#,
        )
        .bind(&rule.title)
        .bind(&rule.category)
        .bind(&rule.subcategory)
        .bind(&rule.content)
        .bind(&rule.source)
        .bind(&rule.page)
        .bind(&rule.id)
        .execute(pool)
        .await?;
        Ok(false)
    } else {
        // Insert new rule
        sqlx::query(
            r#"
            INSERT INTO rules (id, title, category, subcategory, content, source, page, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, datetime('now'), datetime('now'))
            "#,
        )
        .bind(&rule.id)
        .bind(&rule.title)
        .bind(&rule.category)
        .bind(&rule.subcategory)
        .bind(&rule.content)
        .bind(&rule.source)
        .bind(&rule.page)
        .execute(pool)
        .await?;
        Ok(true)
    }
}
