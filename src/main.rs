use axum::Router;
use std::net::SocketAddr;
use tower_http::services::ServeDir;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod routes;
mod models;
mod db;
mod search;
mod ai;

pub use config::Config;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load environment variables
    dotenvy::dotenv().ok();

    // Load configuration
    let config = Config::from_env();
    let port = config.port;

    // Initialize database
    let db_pool = db::init_pool(&config.database_url).await
        .expect("Failed to initialize database");

    // Run migrations
    db::run_migrations(&db_pool).await
        .expect("Failed to run migrations");

    // Build application state
    let state = routes::AppState {
        db: db_pool,
        config,
    };

    // Build application routes
    let app = Router::new()
        .merge(routes::router())
        .nest_service("/static", ServeDir::new("static"))
        .with_state(state);

    // Start server (bind to 0.0.0.0 for Docker compatibility)
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("Server listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
