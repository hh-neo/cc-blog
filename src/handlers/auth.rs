use crate::{
    middleware::auth::create_jwt,
    models::{AuthResponse, CreateUserRequest, LoginRequest, UserResponse},
};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use bcrypt::{hash, verify, DEFAULT_COST};
use serde_json::json;
use sqlx::PgPool;
use validator::Validate;

pub async fn register(
    State(pool): State<PgPool>,
    Json(payload): Json<CreateUserRequest>,
) -> impl IntoResponse {
    if let Err(errors) = payload.validate() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": format!("Validation error: {:?}", errors) })),
        )
            .into_response();
    }

    let password_hash = match hash(&payload.password, DEFAULT_COST) {
        Ok(hash) => hash,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to hash password" })),
            )
                .into_response()
        }
    };

    let user = match sqlx::query_as::<_, crate::models::User>(
        r#"
        INSERT INTO users (username, email, password_hash)
        VALUES ($1, $2, $3)
        RETURNING id, username, email, password_hash, created_at, updated_at
        "#,
    )
    .bind(&payload.username)
    .bind(&payload.email)
    .bind(&password_hash)
    .fetch_one(&pool)
    .await
    {
        Ok(user) => user,
        Err(e) => {
            if e.to_string().contains("duplicate key") {
                return (
                    StatusCode::CONFLICT,
                    Json(json!({ "error": "Username or email already exists" })),
                )
                    .into_response();
            }
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to create user" })),
            )
                .into_response();
        }
    };

    let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "secret".to_string());
    let token = match create_jwt(user.id, &user.username, &jwt_secret) {
        Ok(token) => token,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to create token" })),
            )
                .into_response()
        }
    };

    let response = AuthResponse {
        token,
        user: UserResponse::from(user),
    };

    (StatusCode::CREATED, Json(response)).into_response()
}

pub async fn login(
    State(pool): State<PgPool>,
    Json(payload): Json<LoginRequest>,
) -> impl IntoResponse {
    if let Err(errors) = payload.validate() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": format!("Validation error: {:?}", errors) })),
        )
            .into_response();
    }

    let user = match sqlx::query_as::<_, crate::models::User>(
        r#"
        SELECT id, username, email, password_hash, created_at, updated_at
        FROM users
        WHERE username = $1
        "#,
    )
    .bind(&payload.username)
    .fetch_one(&pool)
    .await
    {
        Ok(user) => user,
        Err(_) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({ "error": "Invalid username or password" })),
            )
                .into_response()
        }
    };

    match verify(&payload.password, &user.password_hash) {
        Ok(true) => {}
        _ => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({ "error": "Invalid username or password" })),
            )
                .into_response()
        }
    }

    let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "secret".to_string());
    let token = match create_jwt(user.id, &user.username, &jwt_secret) {
        Ok(token) => token,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to create token" })),
            )
                .into_response()
        }
    };

    let response = AuthResponse {
        token,
        user: UserResponse::from(user),
    };

    (StatusCode::OK, Json(response)).into_response()
}