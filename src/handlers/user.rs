use crate::models::user::{AuthResponse, LoginInput, RegisterInput, User, UserResponse};
use crate::utils::{jwt::JwtService, AppError, AppResult};
use actix_web::{web, HttpResponse};
use bcrypt::{hash, verify, DEFAULT_COST};
use sqlx::MySqlPool;
use validator::Validate;

pub async fn register(
    pool: web::Data<MySqlPool>,
    jwt_service: web::Data<JwtService>,
    input: web::Json<RegisterInput>,
) -> AppResult<HttpResponse> {
    input.validate().map_err(|e| AppError::ValidationError(e.to_string()))?;

    let existing_user = sqlx::query!(
        "SELECT id FROM users WHERE username = ? OR email = ?",
        input.username,
        input.email
    )
    .fetch_optional(pool.get_ref())
    .await?;

    if existing_user.is_some() {
        return Err(AppError::BadRequest("用户名或邮箱已存在".to_string()));
    }

    let password_hash = hash(&input.password, DEFAULT_COST)?;
    let user = User::new(input.username.clone(), input.email.clone(), password_hash);

    sqlx::query!(
        "INSERT INTO users (id, username, email, password_hash, created_at, updated_at, is_active)
         VALUES (?, ?, ?, ?, ?, ?, ?)",
        user.id,
        user.username,
        user.email,
        user.password_hash,
        user.created_at,
        user.updated_at,
        user.is_active
    )
    .execute(pool.get_ref())
    .await?;

    let token = jwt_service.generate_token(&user.id, &user.username)?;

    Ok(HttpResponse::Created().json(AuthResponse {
        token,
        user: user.to_response(),
    }))
}

pub async fn login(
    pool: web::Data<MySqlPool>,
    jwt_service: web::Data<JwtService>,
    input: web::Json<LoginInput>,
    req: actix_web::HttpRequest,
) -> AppResult<HttpResponse> {
    input.validate().map_err(|e| AppError::ValidationError(e.to_string()))?;

    let user = sqlx::query_as!(
        User,
        "SELECT * FROM users WHERE username = ? OR email = ?",
        input.username_or_email,
        input.username_or_email
    )
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or(AppError::AuthenticationError)?;

    if !user.is_active {
        return Err(AppError::AuthenticationError);
    }

    if !verify(&input.password, &user.password_hash)? {
        log_failed_login(&pool, None, &input.username_or_email, &req).await;
        return Err(AppError::AuthenticationError);
    }

    log_successful_login(&pool, &user.id, &req).await;

    let token = jwt_service.generate_token(&user.id, &user.username)?;

    Ok(HttpResponse::Ok().json(AuthResponse {
        token,
        user: user.to_response(),
    }))
}

pub async fn get_profile(
    pool: web::Data<MySqlPool>,
    auth_user: crate::middleware::AuthUser,
) -> AppResult<HttpResponse> {
    let user = sqlx::query_as!(
        User,
        "SELECT * FROM users WHERE id = ?",
        auth_user.user_id
    )
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or(AppError::NotFoundError)?;

    Ok(HttpResponse::Ok().json(user.to_response()))
}

async fn log_successful_login(pool: &web::Data<MySqlPool>, user_id: &str, req: &actix_web::HttpRequest) {
    let ip_address = req
        .connection_info()
        .peer_addr()
        .unwrap_or("unknown")
        .to_string();

    let user_agent = req
        .headers()
        .get("User-Agent")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown")
        .to_string();

    let log_id = uuid::Uuid::new_v4().to_string();

    let _ = sqlx::query!(
        "INSERT INTO login_logs (id, user_id, ip_address, user_agent, login_status)
         VALUES (?, ?, ?, ?, 'SUCCESS')",
        log_id,
        user_id,
        ip_address,
        user_agent
    )
    .execute(pool.get_ref())
    .await;
}

async fn log_failed_login(pool: &web::Data<MySqlPool>, user_id: Option<&str>, username: &str, req: &actix_web::HttpRequest) {
    let ip_address = req
        .connection_info()
        .peer_addr()
        .unwrap_or("unknown")
        .to_string();

    let user_agent = req
        .headers()
        .get("User-Agent")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown")
        .to_string();

    let log_id = uuid::Uuid::new_v4().to_string();

    let _ = sqlx::query!(
        "INSERT INTO login_logs (id, user_id, username, ip_address, user_agent, login_status)
         VALUES (?, ?, ?, ?, ?, 'FAILED')",
        log_id,
        user_id,
        username,
        ip_address,
        user_agent
    )
    .execute(pool.get_ref())
    .await;
}