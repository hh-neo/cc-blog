use crate::handlers::user::AppState;
use crate::database::{MessageRepository};
use crate::errors::AppError;
use crate::models::{Claims, CreateMessageRequest, MessageListResponse, MessageQuery, MessageResponse, UpdateMessageRequest};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Extension, Json,
};
use uuid::Uuid;
use validator::Validate;

pub async fn create_message(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(request): Json<CreateMessageRequest>,
) -> Result<Json<MessageResponse>, AppError> {
    request.validate()?;

    let user_id = Uuid::parse_str(&claims.sub)?;
    let message = MessageRepository::create_message(&state.db, &user_id, &claims.username, &request).await?;

    Ok(Json(MessageResponse::from(message)))
}

pub async fn get_messages(
    State(state): State<AppState>,
    Query(query): Query<MessageQuery>,
) -> Result<Json<MessageListResponse>, AppError> {
    let response = MessageRepository::list_messages(&state.db, &query).await?;
    Ok(Json(response))
}

pub async fn get_message(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<MessageResponse>, AppError> {
    let message = MessageRepository::find_by_id(&state.db, &id)
        .await?
        .ok_or_else(|| AppError::NotFound("留言不存在".to_string()))?;

    Ok(Json(MessageResponse::from(message)))
}

pub async fn update_message(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateMessageRequest>,
) -> Result<Json<MessageResponse>, AppError> {
    request.validate()?;

    let user_id = Uuid::parse_str(&claims.sub)?;
    let message = MessageRepository::update_message(&state.db, &id, &user_id, &request)
        .await?
        .ok_or_else(|| AppError::NotFound("留言不存在或无权限".to_string()))?;

    Ok(Json(MessageResponse::from(message)))
}

pub async fn delete_message(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let user_id = Uuid::parse_str(&claims.sub)?;
    let deleted = MessageRepository::delete_message(&state.db, &id, &user_id).await?;

    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::NotFound("留言不存在或无权限".to_string()))
    }
}