mod db;
mod handlers;
mod middleware;
mod models;

use axum::{
    routing::{get, post, put, delete},
    Router,
};
use dotenv::dotenv;
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    dotenv().ok();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "message_board=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = db::create_pool(&database_url)
        .await
        .expect("Failed to create database pool");

    // Note: Migrations should be run manually or via a separate migration tool
    // For production, consider using: cargo install sqlx-cli
    // Then run: sqlx migrate run

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Health check route
    let health_route = Router::new()
        .route("/health", get(|| async { "OK" }));

    // Public routes (no authentication required)
    let public_routes = Router::new()
        .route("/api/auth/register", post(handlers::register))
        .route("/api/auth/login", post(handlers::login))
        .route("/api/articles", get(handlers::get_articles))
        .route("/api/articles/:id", get(handlers::get_article));

    // Protected routes (authentication required via JwtAuth extractor)
    let protected_routes = Router::new()
        .route("/api/articles", post(handlers::create_article))
        .route("/api/articles/:id", put(handlers::update_article))
        .route("/api/articles/:id", delete(handlers::delete_article));

    let app = Router::new()
        .merge(health_route)
        .merge(public_routes)
        .merge(protected_routes)
        .with_state(pool.clone())
        .layer(cors)
        .layer(TraceLayer::new_for_http());

    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .expect("Invalid PORT");

    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    println!("Message Board Server running on http://{}", addr);
    println!();
    println!("API Endpoints:");
    println!("  POST   /api/auth/register     - Register new user");
    println!("  POST   /api/auth/login        - Login user");
    println!("  GET    /api/articles          - Get all articles");
    println!("  GET    /api/articles/:id      - Get article by ID");
    println!("  POST   /api/articles          - Create article (auth required)");
    println!("  PUT    /api/articles/:id      - Update article (auth required)");
    println!("  DELETE /api/articles/:id      - Delete article (auth required)");
    println!("  GET    /health                - Health check");

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind");

    axum::serve(listener, app)
        .await
        .expect("Failed to start server");
}
