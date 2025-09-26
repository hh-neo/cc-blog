use axum::{
    http::Method,
    middleware,
    routing::{get, post, put, delete},
    Router,
};
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

use crate::handlers::user::AppState;
use crate::handlers::{message, user};
use crate::middleware::auth_middleware;

pub fn create_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(tower_http::cors::Any)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers(tower_http::cors::Any);

    let public_routes = Router::new()
        .route("/api/auth/register", post(user::register))
        .route("/api/auth/login", post(user::login))
        .route("/api/messages", get(message::get_messages))
        .route("/api/messages/:id", get(message::get_message));

    let protected_routes = Router::new()
        .route("/api/messages", post(message::create_message))
        .route("/api/messages/:id", put(message::update_message))
        .route("/api/messages/:id", delete(message::delete_message))
        .route_layer(middleware::from_fn_with_state(state.clone(), auth_middleware));

    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(cors),
        )
        .with_state(state)
}