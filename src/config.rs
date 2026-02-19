use std::env;

#[derive(Clone)]
pub struct Config {
    pub database_url: String,
    pub claude_api_key: Option<String>,
    pub claude_model: String,
    pub port: u16,
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
        }
    }
}
