use crate::auth::create_jwt;
use crate::db::DbPool;
use crate::models::{AuthResponse, LoginRequest, RegisterRequest, User, UserResponse};
use actix_web::{web, HttpResponse, Result};
use bcrypt::{hash, verify, DEFAULT_COST};

pub async fn register(
    pool: web::Data<DbPool>,
    req: web::Json<RegisterRequest>,
) -> Result<HttpResponse> {
    let password_hash = hash(&req.password, DEFAULT_COST)
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    let result = sqlx::query(
        "INSERT INTO users (username, email, password_hash) VALUES (?, ?, ?)"
    )
    .bind(&req.username)
    .bind(&req.email)
    .bind(&password_hash)
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(result) => {
            let user_id = result.last_insert_id() as i32;

            let user = sqlx::query_as::<_, UserResponse>(
                "SELECT id, username, email, created_at FROM users WHERE id = ?"
            )
            .bind(user_id)
            .fetch_one(pool.get_ref())
            .await
            .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

            let token = create_jwt(user_id, &req.username)
                .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

            Ok(HttpResponse::Created().json(AuthResponse { token, user }))
        }
        Err(sqlx::Error::Database(db_err)) => {
            if db_err.message().contains("Duplicate entry") {
                Ok(HttpResponse::Conflict().json(serde_json::json!({
                    "error": "Username or email already exists"
                })))
            } else {
                Err(actix_web::error::ErrorInternalServerError(db_err))
            }
        }
        Err(e) => Err(actix_web::error::ErrorInternalServerError(e)),
    }
}

pub async fn login(
    pool: web::Data<DbPool>,
    req: web::Json<LoginRequest>,
) -> Result<HttpResponse> {
    let user = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE username = ?"
    )
    .bind(&req.username)
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    match user {
        Some(user) => {
            let valid = verify(&req.password, &user.password_hash)
                .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

            if valid {
                let token = create_jwt(user.id, &user.username)
                    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

                let user_response = UserResponse {
                    id: user.id,
                    username: user.username,
                    email: user.email,
                    created_at: user.created_at,
                };

                Ok(HttpResponse::Ok().json(AuthResponse {
                    token,
                    user: user_response,
                }))
            } else {
                Ok(HttpResponse::Unauthorized().json(serde_json::json!({
                    "error": "Invalid credentials"
                })))
            }
        }
        None => Ok(HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "Invalid credentials"
        }))),
    }
}