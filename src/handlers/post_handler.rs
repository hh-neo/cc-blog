use axum::{
    extract::{Path, State},
    http::StatusCode,
    Extension, Json,
};
use validator::Validate;

use crate::auth::Claims;
use crate::db::DbPool;
use crate::models::{CreatePostRequest, ErrorResponse, PostResponse, UpdatePostRequest};

pub async fn create_post(
    State(pool): State<DbPool>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<CreatePostRequest>,
) -> Result<Json<PostResponse>, (StatusCode, Json<ErrorResponse>)> {
    if let Err(errors) = payload.validate() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new(format!("Validation error: {}", errors))),
        ));
    }

    let result = sqlx::query("INSERT INTO posts (title, content, user_id) VALUES (?, ?, ?)")
        .bind(&payload.title)
        .bind(&payload.content)
        .bind(claims.sub)
        .execute(&pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(format!("Database error: {}", e))),
            )
        })?;

    let post_id = result.last_insert_id() as i32;

    let post: PostResponse = sqlx::query_as::<_, (i32, String, String, i32, String, chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)>(
        "SELECT p.id, p.title, p.content, p.user_id, u.username, p.created_at, p.updated_at
         FROM posts p
         JOIN users u ON p.user_id = u.id
         WHERE p.id = ?"
    )
    .bind(post_id)
    .fetch_one(&pool)
    .await
    .map(|(id, title, content, user_id, username, created_at, updated_at)| PostResponse {
        id,
        title,
        content,
        user_id,
        username,
        created_at,
        updated_at,
    })
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(format!("Database error: {}", e))),
        )
    })?;

    Ok(Json(post))
}

pub async fn get_posts(
    State(pool): State<DbPool>,
) -> Result<Json<Vec<PostResponse>>, (StatusCode, Json<ErrorResponse>)> {
    let posts: Vec<PostResponse> = sqlx::query_as::<_, (i32, String, String, i32, String, chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)>(
        "SELECT p.id, p.title, p.content, p.user_id, u.username, p.created_at, p.updated_at
         FROM posts p
         JOIN users u ON p.user_id = u.id
         ORDER BY p.created_at DESC"
    )
    .fetch_all(&pool)
    .await
    .map(|rows| {
        rows.into_iter()
            .map(|(id, title, content, user_id, username, created_at, updated_at)| PostResponse {
                id,
                title,
                content,
                user_id,
                username,
                created_at,
                updated_at,
            })
            .collect()
    })
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(format!("Database error: {}", e))),
        )
    })?;

    Ok(Json(posts))
}

pub async fn get_post(
    State(pool): State<DbPool>,
    Path(id): Path<i32>,
) -> Result<Json<PostResponse>, (StatusCode, Json<ErrorResponse>)> {
    let post: PostResponse = sqlx::query_as::<_, (i32, String, String, i32, String, chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)>(
        "SELECT p.id, p.title, p.content, p.user_id, u.username, p.created_at, p.updated_at
         FROM posts p
         JOIN users u ON p.user_id = u.id
         WHERE p.id = ?"
    )
    .bind(id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(format!("Database error: {}", e))),
        )
    })?
    .map(|(id, title, content, user_id, username, created_at, updated_at)| PostResponse {
        id,
        title,
        content,
        user_id,
        username,
        created_at,
        updated_at,
    })
    .ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("Post not found")),
        )
    })?;

    Ok(Json(post))
}

pub async fn update_post(
    State(pool): State<DbPool>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<i32>,
    Json(payload): Json<UpdatePostRequest>,
) -> Result<Json<PostResponse>, (StatusCode, Json<ErrorResponse>)> {
    if let Err(errors) = payload.validate() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new(format!("Validation error: {}", errors))),
        ));
    }

    let post: Option<(i32, i32)> = sqlx::query_as("SELECT id, user_id FROM posts WHERE id = ?")
        .bind(id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(format!("Database error: {}", e))),
            )
        })?;

    let (_, user_id) = post.ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("Post not found")),
        )
    })?;

    if user_id != claims.sub {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse::new("You can only update your own posts")),
        ));
    }

    if payload.title.is_none() && payload.content.is_none() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("At least one field must be provided")),
        ));
    }

    let mut query_parts = Vec::new();
    let mut has_title = false;
    let mut has_content = false;

    if payload.title.is_some() {
        query_parts.push("title = ?");
        has_title = true;
    }
    if payload.content.is_some() {
        query_parts.push("content = ?");
        has_content = true;
    }

    let query = format!(
        "UPDATE posts SET {} WHERE id = ?",
        query_parts.join(", ")
    );

    let mut query_builder = sqlx::query(&query);

    if has_title {
        query_builder = query_builder.bind(payload.title.as_ref().unwrap());
    }
    if has_content {
        query_builder = query_builder.bind(payload.content.as_ref().unwrap());
    }

    query_builder = query_builder.bind(id);

    query_builder.execute(&pool).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(format!("Database error: {}", e))),
        )
    })?;

    let post: PostResponse = sqlx::query_as::<_, (i32, String, String, i32, String, chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)>(
        "SELECT p.id, p.title, p.content, p.user_id, u.username, p.created_at, p.updated_at
         FROM posts p
         JOIN users u ON p.user_id = u.id
         WHERE p.id = ?"
    )
    .bind(id)
    .fetch_one(&pool)
    .await
    .map(|(id, title, content, user_id, username, created_at, updated_at)| PostResponse {
        id,
        title,
        content,
        user_id,
        username,
        created_at,
        updated_at,
    })
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(format!("Database error: {}", e))),
        )
    })?;

    Ok(Json(post))
}

pub async fn delete_post(
    State(pool): State<DbPool>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<i32>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let post: Option<(i32, i32)> = sqlx::query_as("SELECT id, user_id FROM posts WHERE id = ?")
        .bind(id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(format!("Database error: {}", e))),
            )
        })?;

    let (_, user_id) = post.ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("Post not found")),
        )
    })?;

    if user_id != claims.sub {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse::new("You can only delete your own posts")),
        ));
    }

    sqlx::query("DELETE FROM posts WHERE id = ?")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(format!("Database error: {}", e))),
            )
        })?;

    Ok(StatusCode::NO_CONTENT)
}