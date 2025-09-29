mod db;
mod handlers;
mod middleware;
mod models;
mod utils;

use actix_cors::Cors;
use actix_web::{middleware::Logger, web, App, HttpResponse, HttpServer};
use dotenv::dotenv;
use env_logger::Env;
use serde_json::json;
use std::env;

async fn health() -> HttpResponse {
    HttpResponse::Ok().json(json!({
        "status": "healthy"
    }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    let database_pool = db::create_pool()
        .await
        .expect("Failed to create database pool");

    let server_host = env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let server_port = env::var("SERVER_PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .expect("SERVER_PORT must be a valid port number");

    println!("Server starting at http://{}:{}", server_host, server_port);

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .app_data(web::Data::new(database_pool.clone()))
            .wrap(cors)
            .wrap(Logger::default())
            .service(
                web::scope("/api")
                    .route("/health", web::get().to(health))
                    // Auth routes
                    .service(
                        web::scope("/auth")
                            .route("/register", web::post().to(handlers::register))
                            .route("/login", web::post().to(handlers::login))
                            .route("/me", web::get().to(handlers::get_current_user)),
                    )
                    // Article routes
                    .service(
                        web::scope("/articles")
                            .route("", web::get().to(handlers::get_articles))
                            .route("", web::post().to(handlers::create_article))
                            .route("/my", web::get().to(handlers::get_user_articles))
                            .route("/{id}", web::get().to(handlers::get_article))
                            .route("/{id}", web::put().to(handlers::update_article))
                            .route("/{id}", web::delete().to(handlers::delete_article)),
                    ),
            )
    })
    .bind((server_host, server_port))?
    .run()
    .await
}
