//! CLI tool for indexing rules into Qdrant vector search.
//!
//! Usage:
//!   cargo run --bin index_vectors
//!   cargo run --bin index_vectors -- --dry-run

use clap::Parser;
use rulecraft::{
    config::VectorSearchConfig,
    search::{
        indexer::index_rules,
        openai_embeddings::OpenAiEmbeddingClient,
        qdrant::QdrantVectorIndex,
    },
    Config,
};

#[derive(Parser, Debug)]
#[command(name = "index_vectors")]
#[command(about = "Index Rulecraft rules into Qdrant using OpenAI embeddings")]
struct Args {
    /// Preview rules that would be indexed without calling OpenAI or Qdrant
    #[arg(long)]
    dry_run: bool,

    /// Stop on the first rule indexing failure
    #[arg(long)]
    fail_fast: bool,

    /// Database URL (default: DATABASE_URL env or sqlite:./rulecraft.db)
    #[arg(short, long)]
    database_url: Option<String>,

    /// Qdrant URL (default: QDRANT_URL env or http://localhost:6333)
    #[arg(long)]
    qdrant_url: Option<String>,

    /// Qdrant collection (default: QDRANT_COLLECTION env or rulecraft_rules_openai_small_v1)
    #[arg(long)]
    qdrant_collection: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    dotenvy::dotenv().ok();

    let args = Args::parse();
    let config = Config::from_env();
    let vector_config = apply_arg_overrides(config.vector, &args);
    let database_url = args
        .database_url
        .clone()
        .unwrap_or(config.database_url);

    println!("Rulecraft Vector Indexer");
    println!("========================");
    println!("Database: {}", database_url);
    println!("Qdrant: {}", vector_config.qdrant_url);
    println!("Collection: {}", vector_config.qdrant_collection);
    println!("Embedding model: {}", vector_config.openai_embedding_model);
    println!();

    let pool = rulecraft::db::init_pool(&database_url).await?;
    rulecraft::db::run_migrations(&pool).await?;
    let rules = rulecraft::db::get_all_rules(&pool).await?;

    if args.dry_run {
        println!("DRY RUN - no embeddings generated and no vectors upserted");
        println!("Rules that would be indexed: {}", rules.len());
        for rule in rules.iter().take(20) {
            println!("  - [{}] {} ({})", rule.category, rule.title, rule.id);
        }
        if rules.len() > 20 {
            println!("  ... and {} more", rules.len() - 20);
        }
        return Ok(());
    }

    let api_key = vector_config
        .openai_api_key
        .clone()
        .ok_or("OPENAI_API_KEY must be set to index vectors")?;

    let embedding_client = OpenAiEmbeddingClient::new(
        api_key,
        vector_config.openai_embedding_model.clone(),
        vector_config.openai_embedding_dimension,
    );
    let vector_index = QdrantVectorIndex::new(
        vector_config.qdrant_url.clone(),
        vector_config.qdrant_collection.clone(),
        vector_config.openai_embedding_dimension,
    );

    let report = index_rules(&rules, &embedding_client, &vector_index, args.fail_fast).await?;

    println!("Index complete");
    println!("  Indexed: {}", report.indexed);
    println!("  Failed:  {}", report.failed.len());
    for failure in report.failed {
        println!("  - {}: {}", failure.rule_id, failure.error);
    }

    Ok(())
}

fn apply_arg_overrides(mut config: VectorSearchConfig, args: &Args) -> VectorSearchConfig {
    if let Some(qdrant_url) = &args.qdrant_url {
        config.qdrant_url = qdrant_url.clone();
    }

    if let Some(qdrant_collection) = &args.qdrant_collection {
        config.qdrant_collection = qdrant_collection.clone();
    }

    config
}
