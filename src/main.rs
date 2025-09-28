use message_board::{auth, db, handlers};
use actix_web::{web, App, HttpServer, middleware};
use actix_web::dev::ServiceRequest;
use actix_web::error::ErrorUnauthorized;
use actix_web::HttpMessage;
use actix_web_httpauth::middleware::HttpAuthentication;
use actix_web_httpauth::extractors::bearer::BearerAuth;
use dotenv::dotenv;
use std::env;

async fn auth_validator(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, (actix_web::Error, ServiceRequest)> {
    match auth::verify_jwt(credentials.token()) {
        Ok(claims) => {
            req.extensions_mut().insert(claims);
            Ok(req)
        }
        Err(_) => Err((ErrorUnauthorized("Invalid token"), req)),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let host = env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("SERVER_PORT").unwrap_or_else(|_| "8080".to_string());

    let pool = db::create_pool()
        .await
        .expect("Failed to create database pool");

    log::info!("Starting server at {}:{}", host, port);
    log::info!("Database: {}", database_url);

    HttpServer::new(move || {
        let auth = HttpAuthentication::bearer(auth_validator);

        App::new()
            .app_data(web::Data::new(pool.clone()))
            .wrap(middleware::Logger::default())
            .service(
                web::scope("/api")
                    .service(
                        web::scope("/auth")
                            .route("/register", web::post().to(handlers::user_handler::register))
                            .route("/login", web::post().to(handlers::user_handler::login))
                    )
                    .service(
                        web::scope("/posts")
                            .route("", web::get().to(handlers::post_handler::get_posts))
                            .route("/{id}", web::get().to(handlers::post_handler::get_post))
                            .service(
                                web::resource("")
                                    .route(web::post().to(handlers::post_handler::create_post))
                                    .wrap(auth.clone())
                            )
                            .service(
                                web::resource("/{id}")
                                    .route(web::put().to(handlers::post_handler::update_post))
                                    .route(web::delete().to(handlers::post_handler::delete_post))
                                    .wrap(auth.clone())
                            )
                    )
            )
    })
    .bind(format!("{}:{}", host, port))?
    .run()
    .await
}