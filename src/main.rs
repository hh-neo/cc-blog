mod auth;
mod config;
mod database;
mod errors;
mod handlers;
mod middleware;
mod models;
mod routes;

use anyhow::Result;
use config::Config;
use database::create_pool;
use dotenv::dotenv;
use handlers::user::AppState;
use routes::create_router;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables
    dotenv().ok();

    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "gold=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = Config::from_env().expect("Failed to load configuration");

    // Create database connection pool
    let pool = create_pool(&config.database_url).await?;

    tracing::info!("Database connection established");

    // Create application state
    let state = AppState {
        db: pool,
        config: config.clone(),
    };

    // Create router
    let app = create_router(state);

    // Start server
    let listener = tokio::net::TcpListener::bind(format!("{}:{}", config.server_host, config.server_port))
        .await?;

    tracing::info!(
        "Server starting on http://{}:{}",
        config.server_host,
        config.server_port
    );

    axum::serve(listener, app).await?;

    Ok(())
}