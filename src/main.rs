mod config;
mod handlers;
mod middleware;
mod models;
mod utils;

use actix_cors::Cors;
use actix_web::{middleware::Logger, web, App, HttpServer};
use dotenv::dotenv;
use std::sync::Arc;

use crate::{
    config::{create_pool, Config},
    handlers::{
        article::{create_article, delete_article, get_article, get_articles, update_article},
        user::{get_profile, login, register},
    },
    middleware::AuthMiddleware,
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init();

    log::info!("Starting Message Board Server...");

    let config = Arc::new(Config::from_env());
    let pool = create_pool()
        .await
        .expect("Failed to create database pool");

    let server_host = config.server_host.clone();
    let server_port = config.server_port;

    log::info!(
        "Server running at http://{}:{}",
        server_host, server_port
    );

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::from(config.clone()))
            .wrap(cors)
            .wrap(Logger::default())
            .service(
                web::scope("/api")
                    .route("/health", web::get().to(health_check))
                    .service(
                        web::scope("/auth")
                            .route("/register", web::post().to(register))
                            .route("/login", web::post().to(login)),
                    )
                    .service(
                        web::scope("/user")
                            .wrap(AuthMiddleware)
                            .route("/profile", web::get().to(get_profile)),
                    )
                    .service(
                        web::scope("/articles")
                            .route("", web::get().to(get_articles))
                            .route("/{id}", web::get().to(get_article))
                            .service(
                                web::scope("")
                                    .wrap(AuthMiddleware)
                                    .route("", web::post().to(create_article))
                                    .route("/{id}", web::put().to(update_article))
                                    .route("/{id}", web::delete().to(delete_article)),
                            ),
                    ),
            )
    })
    .bind((server_host.as_str(), server_port))?
    .run()
    .await
}

async fn health_check() -> &'static str {
    "OK"
}
