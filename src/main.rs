mod config;
mod handlers;
mod middleware;
mod models;
mod utils;

use actix_cors::Cors;
use actix_web::{
    middleware::{Logger, NormalizePath},
    web, App, HttpServer,
};
use actix_web_lab::middleware::from_fn;
use config::Config;
use dotenv::dotenv;
use governor::{Quota, RateLimiter};
use sqlx::mysql::MySqlPoolOptions;
use std::num::NonZeroU32;
use std::sync::Arc;
use utils::jwt::JwtService;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let config = Config::from_env().expect("Failed to load configuration");

    let pool = MySqlPoolOptions::new()
        .max_connections(config.database.max_connections)
        .min_connections(config.database.min_connections)
        .connect(&config.database.url)
        .await
        .expect("Failed to connect to MySQL");

    let jwt_service = web::Data::new(JwtService::new(
        config.jwt.secret.clone(),
        config.jwt.expiration,
    ));

    let rate_limiter = web::Data::new(Arc::new(
        RateLimiter::direct(Quota::per_minute(NonZeroU32::new(60).unwrap()))
    ));

    log::info!(
        "Starting server at http://{}:{}",
        config.server.host,
        config.server.port
    );

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .app_data(jwt_service.clone())
            .app_data(rate_limiter.clone())
            .wrap(Logger::default())
            .wrap(NormalizePath::trim())
            .wrap(
                Cors::default()
                    .allowed_origin_fn(|origin, _req_head| {
                        origin.as_bytes().starts_with(b"http://localhost")
                            || origin.as_bytes().starts_with(b"https://localhost")
                            || origin == "http://127.0.0.1:3000"
                    })
                    .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
                    .allowed_headers(vec![
                        actix_web::http::header::AUTHORIZATION,
                        actix_web::http::header::ACCEPT,
                        actix_web::http::header::CONTENT_TYPE,
                    ])
                    .supports_credentials()
                    .max_age(3600),
            )
            .wrap(from_fn(middleware::auth_middleware))
            .service(
                web::scope("/api")
                    .service(
                        web::scope("/auth")
                            .route("/register", web::post().to(handlers::user::register))
                            .route("/login", web::post().to(handlers::user::login))
                    )
                    .service(
                        web::scope("/users")
                            .wrap(from_fn(middleware::require_auth))
                            .route("/profile", web::get().to(handlers::user::get_profile))
                    )
                    .service(
                        web::scope("/articles")
                            .route("", web::get().to(handlers::article::get_articles))
                            .route("/search", web::get().to(handlers::article::search_articles))
                            .route("/{id}", web::get().to(handlers::article::get_article))
                            .route(
                                "/{id}/comments",
                                web::get().to(handlers::article::get_comments),
                            )
                            .service(
                                web::scope("")
                                    .wrap(from_fn(middleware::require_auth))
                                    .route("", web::post().to(handlers::article::create_article))
                                    .route("/{id}", web::put().to(handlers::article::update_article))
                                    .route(
                                        "/{id}",
                                        web::delete().to(handlers::article::delete_article),
                                    )
                                    .route(
                                        "/{id}/comments",
                                        web::post().to(handlers::article::create_comment),
                                    ),
                            ),
                    )
                    .default_service(web::route().to(not_found)),
            )
            .default_service(web::route().to(not_found))
    })
    .workers(config.server.workers)
    .bind((config.server.host, config.server.port))?
    .run()
    .await
}

async fn not_found() -> actix_web::Result<actix_web::HttpResponse> {
    Ok(actix_web::HttpResponse::NotFound().json(serde_json::json!({
        "error": "Not Found",
        "message": "The requested resource was not found"
    })))
}