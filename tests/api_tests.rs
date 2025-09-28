use axum::{
    http::StatusCode,
    middleware,
    routing::{delete, get, post, put},
    Router,
};
use axum_test::TestServer;
use blog_api::{auth, db, handlers, models};
use serde_json::json;
use uuid::Uuid;

async fn setup_test_server() -> TestServer {
    dotenv::dotenv().ok();

    let pool = db::create_pool()
        .await
        .expect("Failed to create database pool");

    let public_routes = Router::new()
        .route("/register", post(handlers::user_handler::register))
        .route("/login", post(handlers::user_handler::login))
        .route("/posts", get(handlers::post_handler::get_posts))
        .route("/posts/:id", get(handlers::post_handler::get_post))
        .route(
            "/wallets/generate",
            post(handlers::wallet_handler::generate_wallets),
        );

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

    let username = format!("testuser_{}", Uuid::new_v4().to_string().replace("-", ""));
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

    let username = format!("testuser_{}", Uuid::new_v4().to_string().replace("-", ""));
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

    let username = format!("testuser_{}", Uuid::new_v4().to_string().replace("-", ""));
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

    let username = format!("testuser_{}", Uuid::new_v4().to_string().replace("-", ""));
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
}

#[tokio::test]
async fn test_update_post() {
    let server = setup_test_server().await;

    let username = format!("testuser_{}", Uuid::new_v4().to_string().replace("-", ""));
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

    let username = format!("testuser_{}", Uuid::new_v4().to_string().replace("-", ""));
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

    let get_response = server.get(&format!("/posts/{}", created_post.id)).await;

    get_response.assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_generate_single_wallet() {
    let server = setup_test_server().await;

    let response = server
        .post("/wallets/generate")
        .json(&json!({
            "count": 1
        }))
        .await;

    response.assert_status(StatusCode::OK);

    let body: models::GenerateWalletsResponse = response.json();
    assert_eq!(body.count, 1);
    assert_eq!(body.wallets.len(), 1);

    let wallet = &body.wallets[0];
    assert!(wallet.address.starts_with("0x"));
    assert_eq!(wallet.address.len(), 42);
    assert_eq!(wallet.private_key.len(), 64);
}

#[tokio::test]
async fn test_generate_multiple_wallets() {
    let server = setup_test_server().await;

    let response = server
        .post("/wallets/generate")
        .json(&json!({
            "count": 5
        }))
        .await;

    response.assert_status(StatusCode::OK);

    let body: models::GenerateWalletsResponse = response.json();
    assert_eq!(body.count, 5);
    assert_eq!(body.wallets.len(), 5);

    let mut addresses = std::collections::HashSet::new();
    let mut private_keys = std::collections::HashSet::new();

    for wallet in &body.wallets {
        assert!(wallet.address.starts_with("0x"));
        assert_eq!(wallet.address.len(), 42);
        assert_eq!(wallet.private_key.len(), 64);

        addresses.insert(wallet.address.clone());
        private_keys.insert(wallet.private_key.clone());
    }

    assert_eq!(addresses.len(), 5);
    assert_eq!(private_keys.len(), 5);
}

#[tokio::test]
async fn test_generate_maximum_wallets() {
    let server = setup_test_server().await;

    let response = server
        .post("/wallets/generate")
        .json(&json!({
            "count": 10000
        }))
        .await;

    response.assert_status(StatusCode::OK);

    let body: models::GenerateWalletsResponse = response.json();
    assert_eq!(body.count, 10000);
    assert_eq!(body.wallets.len(), 10000);
}

#[tokio::test]
async fn test_generate_wallets_invalid_count_zero() {
    let server = setup_test_server().await;

    let response = server
        .post("/wallets/generate")
        .json(&json!({
            "count": 0
        }))
        .await;

    response.assert_status(StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_generate_wallets_invalid_count_too_large() {
    let server = setup_test_server().await;

    let response = server
        .post("/wallets/generate")
        .json(&json!({
            "count": 10001
        }))
        .await;

    response.assert_status(StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_generate_wallets_missing_count() {
    let server = setup_test_server().await;

    let response = server.post("/wallets/generate").json(&json!({})).await;

    response.assert_status(StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn test_wallet_address_format_validation() {
    let server = setup_test_server().await;

    let response = server
        .post("/wallets/generate")
        .json(&json!({
            "count": 3
        }))
        .await;

    response.assert_status(StatusCode::OK);

    let body: models::GenerateWalletsResponse = response.json();

    for wallet in &body.wallets {
        let address_without_0x = &wallet.address[2..];
        assert!(address_without_0x.chars().all(|c| c.is_ascii_hexdigit()));

        assert!(wallet.private_key.chars().all(|c| c.is_ascii_hexdigit()));
    }
}

#[tokio::test]
async fn test_generate_wallets_with_string_count() {
    let server = setup_test_server().await;

    let response = server
        .post("/wallets/generate")
        .json(&json!({
            "count": "10"
        }))
        .await;

    response.assert_status(StatusCode::OK);

    let body: models::GenerateWalletsResponse = response.json();
    assert_eq!(body.count, 10);
    assert_eq!(body.wallets.len(), 10);
}
