use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use bcrypt::{hash, verify, DEFAULT_COST};
use sqlx::{MySql, Pool, Row};
use uuid::Uuid;
use validator::Validate;

use crate::{
    config::Config,
    models::{AuthResponse, LoginInput, RegisterInput, User, UserResponse},
    utils::{errors::AppError, jwt::{create_token, Claims}},
};

pub async fn register(
    pool: web::Data<Pool<MySql>>,
    config: web::Data<Config>,
    input: web::Json<RegisterInput>,
) -> Result<HttpResponse, AppError> {
    input.validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;

    let existing_user = sqlx::query(
        "SELECT id FROM users WHERE username = ? OR email = ?"
    )
    .bind(&input.username)
    .bind(&input.email)
    .fetch_optional(pool.as_ref())
    .await?;

    if existing_user.is_some() {
        return Err(AppError::BadRequest(
            "Username or email already exists".to_string(),
        ));
    }

    let password_hash = hash(&input.password, DEFAULT_COST)
        .map_err(|_| AppError::InternalServerError)?;

    let user_id = Uuid::new_v4().to_string();

    sqlx::query(
        "INSERT INTO users (id, username, email, password_hash) VALUES (?, ?, ?, ?)"
    )
    .bind(&user_id)
    .bind(&input.username)
    .bind(&input.email)
    .bind(&password_hash)
    .execute(pool.as_ref())
    .await?;

    let row = sqlx::query("SELECT * FROM users WHERE id = ?")
        .bind(&user_id)
        .fetch_one(pool.as_ref())
        .await?;

    let user = User {
        id: row.get("id"),
        username: row.get("username"),
        email: row.get("email"),
        password_hash: row.get("password_hash"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    };

    let token = create_token(&user.id, &user.username, &config.jwt_secret)?;

    let response = AuthResponse {
        user: UserResponse::from(user),
        token,
    };

    Ok(HttpResponse::Created().json(response))
}

pub async fn login(
    pool: web::Data<Pool<MySql>>,
    config: web::Data<Config>,
    input: web::Json<LoginInput>,
) -> Result<HttpResponse, AppError> {
    input.validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;

    let row = sqlx::query("SELECT * FROM users WHERE username = ?")
        .bind(&input.username)
        .fetch_optional(pool.as_ref())
        .await?
        .ok_or(AppError::AuthenticationError)?;

    let user = User {
        id: row.get("id"),
        username: row.get("username"),
        email: row.get("email"),
        password_hash: row.get("password_hash"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    };

    let is_valid = verify(&input.password, &user.password_hash)
        .map_err(|_| AppError::InternalServerError)?;

    if !is_valid {
        return Err(AppError::AuthenticationError);
    }

    let token = create_token(&user.id, &user.username, &config.jwt_secret)?;

    let response = AuthResponse {
        user: UserResponse::from(user),
        token,
    };

    Ok(HttpResponse::Ok().json(response))
}

pub async fn get_profile(
    pool: web::Data<Pool<MySql>>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let extensions = req.extensions();
    let claims = extensions
        .get::<Claims>()
        .ok_or(AppError::AuthenticationError)?
        .clone();

    let row = sqlx::query("SELECT * FROM users WHERE id = ?")
        .bind(&claims.sub)
        .fetch_optional(pool.as_ref())
        .await?
        .ok_or(AppError::NotFound)?;

    let user = User {
        id: row.get("id"),
        username: row.get("username"),
        email: row.get("email"),
        password_hash: row.get("password_hash"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    };

    Ok(HttpResponse::Ok().json(UserResponse::from(user)))
}