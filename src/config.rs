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
        }
    }
}
