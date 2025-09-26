use crate::auth::create_jwt;
use crate::config::Config;
use crate::database::{DbPool, UserRepository};
use crate::errors::AppError;
use crate::models::{AuthResponse, CreateUserRequest, LoginRequest, UserResponse};
use axum::{extract::State, Json};
use validator::Validate;

#[derive(Clone)]
pub struct AppState {
    pub db: DbPool,
    pub config: Config,
}

pub async fn register(
    State(state): State<AppState>,
    Json(request): Json<CreateUserRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    request.validate()?;

    // Check if username already exists
    if UserRepository::find_by_username(&state.db, &request.username)
        .await?
        .is_some()
    {
        return Err(AppError::Conflict("用户名已存在".to_string()));
    }

    // Check if email already exists
    if UserRepository::find_by_email(&state.db, &request.email)
        .await?
        .is_some()
    {
        return Err(AppError::Conflict("邮箱已存在".to_string()));
    }

    let user = UserRepository::create_user(&state.db, &request).await?;
    let token = create_jwt(&user, &state.config)?;

    let response = AuthResponse {
        user: UserResponse::from(user),
        token,
    };

    Ok(Json(response))
}

pub async fn login(
    State(state): State<AppState>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    request.validate()?;

    let user = UserRepository::find_by_username(&state.db, &request.username)
        .await?
        .ok_or_else(|| AppError::Unauthorized("用户名或密码错误".to_string()))?;

    let is_valid = UserRepository::verify_password(&request.password, &user.password_hash).await?;
    if !is_valid {
        return Err(AppError::Unauthorized("用户名或密码错误".to_string()));
    }

    let token = create_jwt(&user, &state.config)?;

    let response = AuthResponse {
        user: UserResponse::from(user),
        token,
    };

    Ok(Json(response))
}