use std::env;

#[derive(Clone)]
pub struct Config {
    pub database_url: String,
    pub claude_api_key: Option<String>,
    pub claude_model: String,
    pub port: u16,
    pub admin_api_key: Option<String>,
    pub ai_rate_limit_per_hour: u32,
    pub search_rate_limit_per_minute: u32,
    pub vector: VectorSearchConfig,
}

#[derive(Clone, Debug)]
pub struct VectorSearchConfig {
    pub enabled: bool,
    pub openai_api_key: Option<String>,
    pub openai_embedding_model: String,
    pub openai_embedding_dimension: usize,
    pub qdrant_url: String,
    pub qdrant_collection: String,
    pub top_k: usize,
    pub score_threshold: f32,
    pub oracle_max_context_rules: usize,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "sqlite:./rulecraft.db".to_string()),
            claude_api_key: env::var("CLAUDE_API_KEY").ok(),
            claude_model: env::var("CLAUDE_MODEL")
                .unwrap_or_else(|_| "claude-sonnet-4-20250514".to_string()),
            port: env::var("PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(3000),
            admin_api_key: env::var("ADMIN_API_KEY").ok(),
            ai_rate_limit_per_hour: env::var("AI_RATE_LIMIT_PER_HOUR")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(5),
            search_rate_limit_per_minute: env::var("SEARCH_RATE_LIMIT_PER_MINUTE")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(30),
            vector: VectorSearchConfig::from_env(),
        }
    }
}

impl VectorSearchConfig {
    pub fn from_env() -> Self {
        Self {
            enabled: env_bool("VECTOR_SEARCH_ENABLED", false),
            openai_api_key: env::var("OPENAI_API_KEY").ok(),
            openai_embedding_model: env::var("OPENAI_EMBEDDING_MODEL")
                .unwrap_or_else(|_| "text-embedding-3-small".to_string()),
            openai_embedding_dimension: env::var("OPENAI_EMBEDDING_DIMENSION")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(1536),
            qdrant_url: env::var("QDRANT_URL")
                .unwrap_or_else(|_| "http://localhost:6333".to_string()),
            qdrant_collection: env::var("QDRANT_COLLECTION")
                .unwrap_or_else(|_| "rulecraft_rules_openai_small_v1".to_string()),
            top_k: env::var("VECTOR_TOP_K")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(10),
            score_threshold: env::var("VECTOR_SCORE_THRESHOLD")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(0.35),
            oracle_max_context_rules: env::var("ORACLE_MAX_CONTEXT_RULES")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(10),
        }
    }
}

fn env_bool(key: &str, default: bool) -> bool {
    env::var(key)
        .ok()
        .map(|value| matches!(value.to_lowercase().as_str(), "1" | "true" | "yes" | "on"))
        .unwrap_or(default)
}
