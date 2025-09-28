use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware,
    routing::{delete, get, post, put},
    Router,
};
use axum_test::TestServer;
use blog_api::{auth, db, handlers, models};
use serde_json::json;

async fn setup_test_server() -> TestServer {
    dotenv::dotenv().ok();

    let pool = db::create_pool()
        .await
        .expect("Failed to create database pool");

    let public_routes = Router::new()
        .route("/register", post(handlers::user_handler::register))
        .route("/login", post(handlers::user_handler::login))
        .route("/posts", get(handlers::post_handler::get_posts))
        .route("/posts/:id", get(handlers::post_handler::get_post));

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
        .with_state(pool);

    TestServer::new(app).unwrap()
}

#[tokio::test]
async fn test_register() {
    let server = setup_test_server().await;

    let username = format!("testuser_{}", chrono::Utc::now().timestamp());
    let response = server
        .post("/register")
        .json(&json!({
            "username": username,
            "email": format!("{}@test.com", username),
            "password": "password123"
        }))
        .await;

    response.assert_status(StatusCode::OK);

    let body: models::AuthResponse = response.json();
    assert!(!body.token.is_empty());
    assert_eq!(body.user.username, username);
}

#[tokio::test]
async fn test_register_duplicate_username() {
    let server = setup_test_server().await;

    let username = format!("testuser_{}", chrono::Utc::now().timestamp());
    server
        .post("/register")
        .json(&json!({
            "username": username,
            "email": format!("{}@test.com", username),
            "password": "password123"
        }))
        .await;

    let response = server
        .post("/register")
        .json(&json!({
            "username": username,
            "email": format!("another_{}@test.com", username),
            "password": "password123"
        }))
        .await;

    response.assert_status(StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_login() {
    let server = setup_test_server().await;

    let username = format!("testuser_{}", chrono::Utc::now().timestamp());
    server
        .post("/register")
        .json(&json!({
            "username": username,
            "email": format!("{}@test.com", username),
            "password": "password123"
        }))
        .await;

    let response = server
        .post("/login")
        .json(&json!({
            "username": username,
            "password": "password123"
        }))
        .await;

    response.assert_status(StatusCode::OK);

    let body: models::AuthResponse = response.json();
    assert!(!body.token.is_empty());
    assert_eq!(body.user.username, username);
}

#[tokio::test]
async fn test_login_invalid_credentials() {
    let server = setup_test_server().await;

    let response = server
        .post("/login")
        .json(&json!({
            "username": "nonexistent",
            "password": "wrongpassword"
        }))
        .await;

    response.assert_status(StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_create_post() {
    let server = setup_test_server().await;

    let username = format!("testuser_{}", chrono::Utc::now().timestamp());
    let register_response = server
        .post("/register")
        .json(&json!({
            "username": username,
            "email": format!("{}@test.com", username),
            "password": "password123"
        }))
        .await;

    let auth: models::AuthResponse = register_response.json();

    let response = server
        .post("/posts")
        .add_header("Authorization", format!("Bearer {}", auth.token))
        .json(&json!({
            "title": "Test Post",
            "content": "This is a test post content"
        }))
        .await;

    response.assert_status(StatusCode::OK);

    let post: models::PostResponse = response.json();
    assert_eq!(post.title, "Test Post");
    assert_eq!(post.content, "This is a test post content");
}

#[tokio::test]
async fn test_create_post_unauthorized() {
    let server = setup_test_server().await;

    let response = server
        .post("/posts")
        .json(&json!({
            "title": "Test Post",
            "content": "This is a test post content"
        }))
        .await;

    response.assert_status(StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_get_posts() {
    let server = setup_test_server().await;

    let response = server.get("/posts").await;

    response.assert_status(StatusCode::OK);

    let posts: Vec<models::PostResponse> = response.json();
    assert!(posts.len() >= 0);
}

#[tokio::test]
async fn test_update_post() {
    let server = setup_test_server().await;

    let username = format!("testuser_{}", chrono::Utc::now().timestamp());
    let register_response = server
        .post("/register")
        .json(&json!({
            "username": username,
            "email": format!("{}@test.com", username),
            "password": "password123"
        }))
        .await;

    let auth: models::AuthResponse = register_response.json();

    let create_response = server
        .post("/posts")
        .add_header("Authorization", format!("Bearer {}", auth.token))
        .json(&json!({
            "title": "Original Title",
            "content": "Original Content"
        }))
        .await;

    let created_post: models::PostResponse = create_response.json();

    let response = server
        .put(&format!("/posts/{}", created_post.id))
        .add_header("Authorization", format!("Bearer {}", auth.token))
        .json(&json!({
            "title": "Updated Title"
        }))
        .await;

    response.assert_status(StatusCode::OK);

    let updated_post: models::PostResponse = response.json();
    assert_eq!(updated_post.title, "Updated Title");
    assert_eq!(updated_post.content, "Original Content");
}

#[tokio::test]
async fn test_delete_post() {
    let server = setup_test_server().await;

    let username = format!("testuser_{}", chrono::Utc::now().timestamp());
    let register_response = server
        .post("/register")
        .json(&json!({
            "username": username,
            "email": format!("{}@test.com", username),
            "password": "password123"
        }))
        .await;

    let auth: models::AuthResponse = register_response.json();

    let create_response = server
        .post("/posts")
        .add_header("Authorization", format!("Bearer {}", auth.token))
        .json(&json!({
            "title": "To Be Deleted",
            "content": "This post will be deleted"
        }))
        .await;

    let created_post: models::PostResponse = create_response.json();

    let response = server
        .delete(&format!("/posts/{}", created_post.id))
        .add_header("Authorization", format!("Bearer {}", auth.token))
        .await;

    response.assert_status(StatusCode::NO_CONTENT);

    let get_response = server
        .get(&format!("/posts/{}", created_post.id))
        .await;

    get_response.assert_status(StatusCode::NOT_FOUND);
}