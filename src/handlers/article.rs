use actix_web::{web, HttpResponse};
use sqlx::{PgPool, FromRow};
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::{
    middleware::AuthUser,
    models::{Article, ArticleResponse, CreateArticleInput, UpdateArticleInput},
    utils::AppError,
};

#[derive(Debug, FromRow)]
struct ArticleWithUsername {
    id: Uuid,
    title: String,
    content: String,
    author_id: Uuid,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    username: String,
}

pub async fn create_article(
    pool: web::Data<PgPool>,
    auth_user: AuthUser,
    input: web::Json<CreateArticleInput>,
) -> Result<HttpResponse, AppError> {
    let article = sqlx::query_as::<_, Article>(
        r#"
        INSERT INTO articles (title, content, author_id)
        VALUES ($1, $2, $3)
        RETURNING *
        "#,
    )
    .bind(&input.title)
    .bind(&input.content)
    .bind(auth_user.user_id)
    .fetch_one(pool.get_ref())
    .await?;

    let article_response = ArticleResponse {
        id: article.id,
        title: article.title,
        content: article.content,
        author_id: article.author_id,
        author_username: Some(auth_user.username),
        created_at: article.created_at,
        updated_at: article.updated_at,
    };

    Ok(HttpResponse::Created().json(article_response))
}

pub async fn get_articles(pool: web::Data<PgPool>) -> Result<HttpResponse, AppError> {
    let articles = sqlx::query_as::<_, ArticleWithUsername>(
        r#"
        SELECT a.id, a.title, a.content, a.author_id, a.created_at, a.updated_at, u.username
        FROM articles a
        JOIN users u ON a.author_id = u.id
        ORDER BY a.created_at DESC
        "#,
    )
    .fetch_all(pool.get_ref())
    .await?;

    let article_responses: Vec<ArticleResponse> = articles
        .into_iter()
        .map(|article| ArticleResponse {
            id: article.id,
            title: article.title,
            content: article.content,
            author_id: article.author_id,
            author_username: Some(article.username),
            created_at: article.created_at,
            updated_at: article.updated_at,
        })
        .collect();

    Ok(HttpResponse::Ok().json(article_responses))
}

pub async fn get_article(
    pool: web::Data<PgPool>,
    article_id: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let article = sqlx::query_as::<_, ArticleWithUsername>(
        r#"
        SELECT a.id, a.title, a.content, a.author_id, a.created_at, a.updated_at, u.username
        FROM articles a
        JOIN users u ON a.author_id = u.id
        WHERE a.id = $1
        "#,
    )
    .bind(article_id.into_inner())
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or(AppError::ArticleNotFound)?;

    let article_response = ArticleResponse {
        id: article.id,
        title: article.title,
        content: article.content,
        author_id: article.author_id,
        author_username: Some(article.username),
        created_at: article.created_at,
        updated_at: article.updated_at,
    };

    Ok(HttpResponse::Ok().json(article_response))
}

pub async fn update_article(
    pool: web::Data<PgPool>,
    auth_user: AuthUser,
    article_id: web::Path<Uuid>,
    input: web::Json<UpdateArticleInput>,
) -> Result<HttpResponse, AppError> {
    let article_id = article_id.into_inner();

    // Check if the article exists and belongs to the user
    let existing_article = sqlx::query_as::<_, Article>(
        r#"
        SELECT * FROM articles WHERE id = $1 AND author_id = $2
        "#,
    )
    .bind(article_id)
    .bind(auth_user.user_id)
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or(AppError::ArticleNotFound)?;

    // Build dynamic update query
    let title = input.title.as_ref().unwrap_or(&existing_article.title);
    let content = input.content.as_ref().unwrap_or(&existing_article.content);

    let updated_article = sqlx::query_as::<_, Article>(
        r#"
        UPDATE articles
        SET title = $1, content = $2, updated_at = NOW()
        WHERE id = $3
        RETURNING *
        "#,
    )
    .bind(title)
    .bind(content)
    .bind(article_id)
    .fetch_one(pool.get_ref())
    .await?;

    let article_response = ArticleResponse {
        id: updated_article.id,
        title: updated_article.title,
        content: updated_article.content,
        author_id: updated_article.author_id,
        author_username: Some(auth_user.username),
        created_at: updated_article.created_at,
        updated_at: updated_article.updated_at,
    };

    Ok(HttpResponse::Ok().json(article_response))
}

pub async fn delete_article(
    pool: web::Data<PgPool>,
    auth_user: AuthUser,
    article_id: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let result = sqlx::query(
        r#"
        DELETE FROM articles
        WHERE id = $1 AND author_id = $2
        "#,
    )
    .bind(article_id.into_inner())
    .bind(auth_user.user_id)
    .execute(pool.get_ref())
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::ArticleNotFound);
    }

    Ok(HttpResponse::NoContent().finish())
}

pub async fn get_user_articles(
    pool: web::Data<PgPool>,
    auth_user: AuthUser,
) -> Result<HttpResponse, AppError> {
    let articles = sqlx::query_as::<_, Article>(
        r#"
        SELECT * FROM articles
        WHERE author_id = $1
        ORDER BY created_at DESC
        "#,
    )
    .bind(auth_user.user_id)
    .fetch_all(pool.get_ref())
    .await?;

    let article_responses: Vec<ArticleResponse> = articles
        .into_iter()
        .map(|article| ArticleResponse {
            id: article.id,
            title: article.title,
            content: article.content,
            author_id: article.author_id,
            author_username: Some(auth_user.username.clone()),
            created_at: article.created_at,
            updated_at: article.updated_at,
        })
        .collect();

    Ok(HttpResponse::Ok().json(article_responses))
}