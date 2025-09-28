use axum::{extract::State, http::StatusCode, Json};
use validator::Validate;

use crate::auth::{create_token, hash_password, verify_password};
use crate::db::DbPool;
use crate::models::{AuthResponse, ErrorResponse, LoginRequest, RegisterRequest, User, UserResponse};

pub async fn register(
    State(pool): State<DbPool>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, Json<ErrorResponse>)> {
    if let Err(errors) = payload.validate() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new(format!("Validation error: {}", errors))),
        ));
    }

    let existing_user: Option<(i32,)> =
        sqlx::query_as("SELECT id FROM users WHERE username = ? OR email = ?")
            .bind(&payload.username)
            .bind(&payload.email)
            .fetch_optional(&pool)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse::new(format!("Database error: {}", e))),
                )
            })?;

    if existing_user.is_some() {
        return Err((
            StatusCode::CONFLICT,
            Json(ErrorResponse::new("Username or email already exists")),
        ));
    }

    let password_hash = hash_password(&payload.password).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(format!("Password hashing error: {}", e))),
        )
    })?;

    let result = sqlx::query(
        "INSERT INTO users (username, email, password_hash) VALUES (?, ?, ?)"
    )
    .bind(&payload.username)
    .bind(&payload.email)
    .bind(&password_hash)
    .execute(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(format!("Database error: {}", e))),
        )
    })?;

    let user_id = result.last_insert_id() as i32;

    let token = create_token(user_id, &payload.username).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(format!("Token creation error: {}", e))),
        )
    })?;

    Ok(Json(AuthResponse {
        token,
        user: UserResponse {
            id: user_id,
            username: payload.username,
            email: payload.email,
        },
    }))
}

pub async fn login(
    State(pool): State<DbPool>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, Json<ErrorResponse>)> {
    if let Err(errors) = payload.validate() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new(format!("Validation error: {}", errors))),
        ));
    }

    let user: Option<User> = sqlx::query_as::<_, User>(
        "SELECT id, username, email, password_hash, created_at, updated_at FROM users WHERE username = ?"
    )
    .bind(&payload.username)
    .fetch_optional(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(format!("Database error: {}", e))),
        )
    })?;

    let user = user.ok_or_else(|| {
        (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse::new("Invalid username or password")),
        )
    })?;

    let valid = verify_password(&payload.password, &user.password_hash).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(format!("Password verification error: {}", e))),
        )
    })?;

    if !valid {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse::new("Invalid username or password")),
        ));
    }

    let token = create_token(user.id, &user.username).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(format!("Token creation error: {}", e))),
        )
    })?;

    Ok(Json(AuthResponse {
        token,
        user: user.into(),
    }))
}