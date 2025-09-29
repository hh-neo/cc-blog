use actix_web::{web, HttpResponse};
use sqlx::PgPool;

use crate::{
    models::{AuthResponse, LoginInput, RegisterInput, User, UserResponse},
    utils::{create_jwt, hash_password, verify_password, AppError},
};

pub async fn register(
    pool: web::Data<PgPool>,
    input: web::Json<RegisterInput>,
) -> Result<HttpResponse, AppError> {
    let password_hash = hash_password(&input.password)?;

    let user = sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (username, email, password_hash)
        VALUES ($1, $2, $3)
        RETURNING *
        "#,
    )
    .bind(&input.username)
    .bind(&input.email)
    .bind(&password_hash)
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| match e {
        sqlx::Error::Database(ref db_err) if db_err.constraint() == Some("users_username_key") => {
            AppError::DuplicateUser
        }
        sqlx::Error::Database(ref db_err) if db_err.constraint() == Some("users_email_key") => {
            AppError::DuplicateUser
        }
        _ => AppError::DatabaseError(e),
    })?;

    let token = create_jwt(&user.id.to_string(), &user.username)?;

    Ok(HttpResponse::Created().json(AuthResponse {
        token,
        user: user.into(),
    }))
}

pub async fn login(
    pool: web::Data<PgPool>,
    input: web::Json<LoginInput>,
) -> Result<HttpResponse, AppError> {
    let user = sqlx::query_as::<_, User>(
        r#"
        SELECT * FROM users WHERE username = $1
        "#,
    )
    .bind(&input.username)
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or(AppError::AuthenticationError)?;

    if !verify_password(&input.password, &user.password_hash)? {
        return Err(AppError::AuthenticationError);
    }

    let token = create_jwt(&user.id.to_string(), &user.username)?;

    Ok(HttpResponse::Ok().json(AuthResponse {
        token,
        user: user.into(),
    }))
}

pub async fn get_current_user(
    pool: web::Data<PgPool>,
    auth_user: crate::middleware::AuthUser,
) -> Result<HttpResponse, AppError> {
    let user = sqlx::query_as::<_, User>(
        r#"
        SELECT * FROM users WHERE id = $1
        "#,
    )
    .bind(auth_user.user_id)
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or(AppError::UserNotFound)?;

    let user_response: UserResponse = user.into();
    Ok(HttpResponse::Ok().json(user_response))
}