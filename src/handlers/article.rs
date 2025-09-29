use crate::{
    middleware::JwtAuth,
    models::{Article, ArticleResponse, CreateArticleRequest, UpdateArticleRequest},
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;
use sqlx::{PgPool, Row};
use uuid::Uuid;
use validator::Validate;

pub async fn create_article(
    State(pool): State<PgPool>,
    auth: JwtAuth,
    Json(payload): Json<CreateArticleRequest>,
) -> impl IntoResponse {
    if let Err(errors) = payload.validate() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": format!("Validation error: {:?}", errors) })),
        )
            .into_response();
    }

    let article = match sqlx::query_as::<_, Article>(
        r#"
        INSERT INTO articles (user_id, title, content)
        VALUES ($1, $2, $3)
        RETURNING id, user_id, title, content, created_at, updated_at
        "#,
    )
    .bind(auth.user_id)
    .bind(&payload.title)
    .bind(&payload.content)
    .fetch_one(&pool)
    .await
    {
        Ok(article) => article,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to create article" })),
            )
                .into_response()
        }
    };

    (StatusCode::CREATED, Json(ArticleResponse::from(article))).into_response()
}

pub async fn get_articles(State(pool): State<PgPool>) -> impl IntoResponse {
    let articles = match sqlx::query_as::<_, Article>(
        r#"
        SELECT id, user_id, title, content, created_at, updated_at
        FROM articles
        ORDER BY created_at DESC
        "#
    )
    .fetch_all(&pool)
    .await
    {
        Ok(articles) => articles,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to fetch articles" })),
            )
                .into_response()
        }
    };

    let response: Vec<ArticleResponse> = articles.into_iter().map(ArticleResponse::from).collect();
    (StatusCode::OK, Json(response)).into_response()
}

pub async fn get_article(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let article = match sqlx::query_as::<_, Article>(
        r#"
        SELECT id, user_id, title, content, created_at, updated_at
        FROM articles
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_one(&pool)
    .await
    {
        Ok(article) => article,
        Err(_) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "Article not found" })),
            )
                .into_response()
        }
    };

    (StatusCode::OK, Json(ArticleResponse::from(article))).into_response()
}

pub async fn update_article(
    State(pool): State<PgPool>,
    auth: JwtAuth,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateArticleRequest>,
) -> impl IntoResponse {
    if let Err(errors) = payload.validate() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": format!("Validation error: {:?}", errors) })),
        )
            .into_response();
    }

    // First check if the article exists and belongs to the user
    let existing_article = match sqlx::query(
        r#"
        SELECT user_id
        FROM articles
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_one(&pool)
    .await
    {
        Ok(article) => article,
        Err(_) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "Article not found" })),
            )
                .into_response()
        }
    };

    let user_id: Uuid = existing_article.try_get("user_id").unwrap();
    if user_id != auth.user_id {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "You don't have permission to update this article" })),
        )
            .into_response();
    }

    let article = if payload.title.is_some() && payload.content.is_some() {
        sqlx::query_as::<_, Article>(
            r#"
            UPDATE articles
            SET title = $2, content = $3, updated_at = NOW()
            WHERE id = $1
            RETURNING id, user_id, title, content, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(payload.title.unwrap())
        .bind(payload.content.unwrap())
        .fetch_one(&pool)
        .await
    } else if payload.title.is_some() {
        sqlx::query_as::<_, Article>(
            r#"
            UPDATE articles
            SET title = $2, updated_at = NOW()
            WHERE id = $1
            RETURNING id, user_id, title, content, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(payload.title.unwrap())
        .fetch_one(&pool)
        .await
    } else if payload.content.is_some() {
        sqlx::query_as::<_, Article>(
            r#"
            UPDATE articles
            SET content = $2, updated_at = NOW()
            WHERE id = $1
            RETURNING id, user_id, title, content, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(payload.content.unwrap())
        .fetch_one(&pool)
        .await
    } else {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "No fields to update" })),
        )
            .into_response();
    };

    let article = match article
    {
        Ok(article) => article,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to update article" })),
            )
                .into_response()
        }
    };

    (StatusCode::OK, Json(ArticleResponse::from(article))).into_response()
}

pub async fn delete_article(
    State(pool): State<PgPool>,
    auth: JwtAuth,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    // First check if the article exists and belongs to the user
    let existing_article = match sqlx::query(
        r#"
        SELECT user_id
        FROM articles
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_one(&pool)
    .await
    {
        Ok(article) => article,
        Err(_) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "Article not found" })),
            )
                .into_response()
        }
    };

    let user_id: Uuid = existing_article.try_get("user_id").unwrap();
    if user_id != auth.user_id {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "You don't have permission to delete this article" })),
        )
            .into_response();
    }

    match sqlx::query(
        r#"
        DELETE FROM articles
        WHERE id = $1
        "#,
    )
    .bind(id)
    .execute(&pool)
    .await
    {
        Ok(_) => (StatusCode::NO_CONTENT, Json(json!({}))).into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Failed to delete article" })),
        )
            .into_response(),
    }
}