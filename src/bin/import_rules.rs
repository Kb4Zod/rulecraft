//! CLI tool for importing D&D rules from YAML files into the database
//!
//! Usage:
//!   cargo run --bin import_rules           # Import all rules
//!   cargo run --bin import_rules -- --dry-run  # Preview without changes

use clap::Parser;
use glob::glob;
use serde::Deserialize;
use std::path::PathBuf;

/// Import D&D rules from YAML files into the database
#[derive(Parser, Debug)]
#[command(name = "import_rules")]
#[command(about = "Import D&D 2024 rules from YAML files into the SQLite database")]
struct Args {
    /// Preview changes without modifying the database
    #[arg(long)]
    dry_run: bool,

    /// Path to rules directory (default: data/rules)
    #[arg(short, long, default_value = "data/rules")]
    rules_dir: PathBuf,

    /// Database URL (default: from DATABASE_URL env or sqlite:./rulecraft.db)
    #[arg(short, long)]
    database_url: Option<String>,
}

/// Structure for a single rule in YAML
#[derive(Debug, Deserialize)]
struct YamlRule {
    id: String,
    title: String,
    #[serde(default)]
    subcategory: Option<String>,
    #[serde(default)]
    page: Option<i32>,
    content: String,
}

/// Structure for a YAML rules file
#[derive(Debug, Deserialize)]
struct RulesFile {
    category: String,
    source: String,
    rules: Vec<YamlRule>,
}

/// Rule struct matching the database schema
struct Rule {
    id: String,
    title: String,
    category: String,
    subcategory: Option<String>,
    content: String,
    source: String,
    page: Option<i32>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    // Load environment variables
    dotenvy::dotenv().ok();

    // Get database URL
    let database_url = args
        .database_url
        .or_else(|| std::env::var("DATABASE_URL").ok())
        .unwrap_or_else(|| "sqlite:./rulecraft.db".to_string());

    println!("Rulecraft Rules Importer");
    println!("========================");
    println!();

    if args.dry_run {
        println!("DRY RUN MODE - No changes will be made");
        println!();
    }

    // Find YAML files
    let pattern = args.rules_dir.join("*.yaml");
    let pattern_str = pattern.to_string_lossy();

    let yaml_files: Vec<PathBuf> = glob(&pattern_str)?
        .filter_map(|entry| entry.ok())
        .collect();

    if yaml_files.is_empty() {
        println!("No YAML files found in {:?}", args.rules_dir);
        println!("Expected files like: combat.yaml, conditions.yaml, etc.");
        return Ok(());
    }

    println!("Found {} YAML file(s):", yaml_files.len());
    for file in &yaml_files {
        println!("  - {}", file.display());
    }
    println!();

    // Parse all YAML files
    let mut all_rules: Vec<Rule> = Vec::new();
    let mut parse_errors = 0;

    for yaml_path in &yaml_files {
        let content = std::fs::read_to_string(yaml_path)?;

        match serde_yaml::from_str::<RulesFile>(&content) {
            Ok(rules_file) => {
                println!(
                    "Parsed {}: {} rules in category '{}'",
                    yaml_path.file_name().unwrap().to_string_lossy(),
                    rules_file.rules.len(),
                    rules_file.category
                );

                for yaml_rule in rules_file.rules {
                    all_rules.push(Rule {
                        id: yaml_rule.id,
                        title: yaml_rule.title,
                        category: rules_file.category.clone(),
                        subcategory: yaml_rule.subcategory,
                        content: yaml_rule.content,
                        source: rules_file.source.clone(),
                        page: yaml_rule.page,
                    });
                }
            }
            Err(e) => {
                eprintln!("Error parsing {}: {}", yaml_path.display(), e);
                parse_errors += 1;
            }
        }
    }

    println!();
    println!("Total rules parsed: {}", all_rules.len());

    if parse_errors > 0 {
        eprintln!("Warning: {} file(s) had parse errors", parse_errors);
    }

    if args.dry_run {
        println!();
        println!("Rules that would be imported:");
        for rule in &all_rules {
            println!(
                "  [{}/{}] {} (id: {})",
                rule.category,
                rule.subcategory.as_deref().unwrap_or("-"),
                rule.title,
                rule.id
            );
        }
        println!();
        println!("Dry run complete. Use without --dry-run to import.");
        return Ok(());
    }

    // Connect to database
    println!();
    println!("Connecting to database...");

    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect(&database_url)
        .await?;

    // Run migrations to ensure schema exists
    rulecraft::db::run_migrations(&pool).await?;

    // Import rules
    let mut inserted = 0;
    let mut updated = 0;

    for rule in &all_rules {
        // Convert to the crate's Rule type
        let db_rule = rulecraft::models::Rule {
            id: rule.id.clone(),
            title: rule.title.clone(),
            category: rule.category.clone(),
            subcategory: rule.subcategory.clone(),
            content: rule.content.clone(),
            source: rule.source.clone(),
            page: rule.page,
            created_at: String::new(), // Will be set by upsert
            updated_at: String::new(), // Will be set by upsert
        };

        match rulecraft::db::upsert_rule(&pool, &db_rule).await {
            Ok(is_new) => {
                if is_new {
                    inserted += 1;
                } else {
                    updated += 1;
                }
            }
            Err(e) => {
                eprintln!("Error upserting rule '{}': {}", rule.id, e);
            }
        }
    }

    println!();
    println!("Import complete!");
    println!("  Inserted: {} new rules", inserted);
    println!("  Updated:  {} existing rules", updated);
    println!("  Total:    {} rules in database", inserted + updated);

    Ok(())
}
