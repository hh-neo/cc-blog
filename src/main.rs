use axum::{
    middleware,
    routing::{delete, get, post, put},
    Router,
};
use dotenv::dotenv;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use blog_api::{auth, db, handlers};

#[tokio::main]
async fn main() {
    dotenv().ok();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "blog_api=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let pool = db::create_pool()
        .await
        .expect("Failed to create database pool");

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let public_routes = Router::new()
        .route("/register", post(handlers::user_handler::register))
        .route("/login", post(handlers::user_handler::login))
        .route("/posts", get(handlers::post_handler::get_posts))
        .route("/posts/:id", get(handlers::post_handler::get_post))
        .route("/wallets/generate", post(handlers::wallet_handler::generate_wallets))
        .route("/transfer/batch", post(handlers::transfer_handler::batch_transfer));

    let protected_routes = Router::new()
        .route("/posts", post(handlers::post_handler::create_post))
        .route("/posts/:id", put(handlers::post_handler::update_post))
        .route("/posts/:id", delete(handlers::post_handler::delete_post))
        .route_layer(middleware::from_fn_with_state(
            pool.clone(),
            auth::auth_middleware,
        ));

    let app = Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(pool);

    let addr = std::env::var("SERVER_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:3000".to_string());

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind address");

    tracing::info!("Server running on {}", addr);

    axum::serve(listener, app)
        .await
        .expect("Failed to start server");
}