use actix_web::{test, web, App};
use message_board::{auth, db, handlers, models};
use serde_json::json;
use dotenv::dotenv;

async fn setup_test_pool() -> db::DbPool {
    dotenv().ok();
    db::create_pool().await.expect("Failed to create test pool")
}

#[actix_web::test]
async fn test_register_user() {
    let pool = setup_test_pool().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .service(
                web::scope("/api/auth")
                    .route("/register", web::post().to(handlers::user_handler::register))
            )
    ).await;

    let unique_username = format!("testuser_{}", chrono::Utc::now().timestamp());
    let unique_email = format!("{}@test.com", unique_username);

    let req = test::TestRequest::post()
        .uri("/api/auth/register")
        .set_json(json!({
            "username": unique_username,
            "email": unique_email,
            "password": "password123"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);
}

#[actix_web::test]
async fn test_login_user() {
    let pool = setup_test_pool().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .service(
                web::scope("/api/auth")
                    .route("/register", web::post().to(handlers::user_handler::register))
                    .route("/login", web::post().to(handlers::user_handler::login))
            )
    ).await;

    let unique_username = format!("logintest_{}", chrono::Utc::now().timestamp());
    let unique_email = format!("{}@test.com", unique_username);

    let register_req = test::TestRequest::post()
        .uri("/api/auth/register")
        .set_json(json!({
            "username": unique_username,
            "email": unique_email,
            "password": "password123"
        }))
        .to_request();

    test::call_service(&app, register_req).await;

    let login_req = test::TestRequest::post()
        .uri("/api/auth/login")
        .set_json(json!({
            "username": unique_username,
            "password": "password123"
        }))
        .to_request();

    let resp = test::call_service(&app, login_req).await;
    assert_eq!(resp.status(), 200);
}

#[actix_web::test]
async fn test_create_post() {
    let pool = setup_test_pool().await;

    let unique_username = format!("posttest_{}", chrono::Utc::now().timestamp());
    let unique_email = format!("{}@test.com", unique_username);

    let password_hash = bcrypt::hash("password123", bcrypt::DEFAULT_COST).unwrap();
    let result = sqlx::query(
        "INSERT INTO users (username, email, password_hash) VALUES (?, ?, ?)"
    )
    .bind(&unique_username)
    .bind(&unique_email)
    .bind(&password_hash)
    .execute(&pool)
    .await
    .unwrap();

    let user_id = result.last_insert_id() as i32;
    let token = auth::create_jwt(user_id, &unique_username).unwrap();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .service(
                web::scope("/api/posts")
                    .route("", web::post().to(handlers::post_handler::create_post))
            )
    ).await;

    let req = test::TestRequest::post()
        .uri("/api/posts")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(json!({
            "title": "Test Post",
            "content": "This is a test post"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);
}

#[actix_web::test]
async fn test_get_posts() {
    let pool = setup_test_pool().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .service(
                web::scope("/api/posts")
                    .route("", web::get().to(handlers::post_handler::get_posts))
            )
    ).await;

    let req = test::TestRequest::get()
        .uri("/api/posts")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);
}

#[test]
fn test_jwt_creation_and_verification() {
    std::env::set_var("JWT_SECRET", "test_secret_key");

    let user_id = 1;
    let username = "testuser";

    let token = auth::create_jwt(user_id, username).unwrap();
    let claims = auth::verify_jwt(&token).unwrap();

    assert_eq!(claims.sub, user_id);
    assert_eq!(claims.username, username);
}

#[test]
fn test_jwt_invalid_token() {
    std::env::set_var("JWT_SECRET", "test_secret_key");

    let result = auth::verify_jwt("invalid_token");
    assert!(result.is_err());
}