use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use sqlx::{MySql, Pool, Row};
use uuid::Uuid;
use validator::Validate;

use crate::{
    models::{Article, ArticleQuery, ArticleResponse, CreateArticleInput, PaginatedResponse, UpdateArticleInput},
    utils::{errors::AppError, jwt::Claims},
};

pub async fn create_article(
    pool: web::Data<Pool<MySql>>,
    input: web::Json<CreateArticleInput>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let extensions = req.extensions();
    let claims = extensions
        .get::<Claims>()
        .ok_or(AppError::AuthenticationError)?
        .clone();

    input.validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;

    let article_id = Uuid::new_v4().to_string();

    sqlx::query(
        "INSERT INTO articles (id, user_id, title, content) VALUES (?, ?, ?, ?)"
    )
    .bind(&article_id)
    .bind(&claims.sub)
    .bind(&input.title)
    .bind(&input.content)
    .execute(pool.as_ref())
    .await?;

    let row = sqlx::query("SELECT * FROM articles WHERE id = ?")
        .bind(&article_id)
        .fetch_one(pool.as_ref())
        .await?;

    let article = Article {
        id: row.get("id"),
        user_id: row.get("user_id"),
        title: row.get("title"),
        content: row.get("content"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    };

    Ok(HttpResponse::Created().json(article))
}

pub async fn get_articles(
    pool: web::Data<Pool<MySql>>,
    query: web::Query<ArticleQuery>,
) -> Result<HttpResponse, AppError> {
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(10).min(100).max(1);
    let offset = (page - 1) * per_page;

    let total: i64 = if let Some(user_id) = &query.user_id {
        let row = sqlx::query("SELECT COUNT(*) as count FROM articles WHERE user_id = ?")
            .bind(user_id)
            .fetch_one(pool.as_ref())
            .await?;
        row.get(0)
    } else {
        let row = sqlx::query("SELECT COUNT(*) as count FROM articles")
            .fetch_one(pool.as_ref())
            .await?;
        row.get(0)
    };

    let articles = if let Some(user_id) = &query.user_id {
        sqlx::query(
            r#"
            SELECT
                a.id, a.user_id, a.title, a.content,
                a.created_at, a.updated_at, u.username
            FROM articles a
            JOIN users u ON a.user_id = u.id
            WHERE a.user_id = ?
            ORDER BY a.created_at DESC
            LIMIT ? OFFSET ?
            "#
        )
        .bind(user_id)
        .bind(per_page)
        .bind(offset)
        .fetch_all(pool.as_ref())
        .await?
    } else {
        sqlx::query(
            r#"
            SELECT
                a.id, a.user_id, a.title, a.content,
                a.created_at, a.updated_at, u.username
            FROM articles a
            JOIN users u ON a.user_id = u.id
            ORDER BY a.created_at DESC
            LIMIT ? OFFSET ?
            "#
        )
        .bind(per_page)
        .bind(offset)
        .fetch_all(pool.as_ref())
        .await?
    };

    let article_responses: Vec<ArticleResponse> = articles
        .into_iter()
        .map(|row| ArticleResponse {
            id: row.get("id"),
            user_id: row.get("user_id"),
            username: row.get("username"),
            title: row.get("title"),
            content: row.get("content"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
        .collect();

    let response = PaginatedResponse {
        data: article_responses,
        total,
        page,
        per_page,
    };

    Ok(HttpResponse::Ok().json(response))
}

pub async fn get_article(
    pool: web::Data<Pool<MySql>>,
    article_id: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    let article = sqlx::query(
        r#"
        SELECT
            a.id, a.user_id, a.title, a.content,
            a.created_at, a.updated_at, u.username
        FROM articles a
        JOIN users u ON a.user_id = u.id
        WHERE a.id = ?
        "#
    )
    .bind(article_id.as_str())
    .fetch_optional(pool.as_ref())
    .await?
    .ok_or(AppError::NotFound)?;

    let response = ArticleResponse {
        id: article.get("id"),
        user_id: article.get("user_id"),
        username: article.get("username"),
        title: article.get("title"),
        content: article.get("content"),
        created_at: article.get("created_at"),
        updated_at: article.get("updated_at"),
    };

    Ok(HttpResponse::Ok().json(response))
}

pub async fn update_article(
    pool: web::Data<Pool<MySql>>,
    article_id: web::Path<String>,
    input: web::Json<UpdateArticleInput>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let extensions = req.extensions();
    let claims = extensions
        .get::<Claims>()
        .ok_or(AppError::AuthenticationError)?
        .clone();

    input.validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;

    let article = sqlx::query("SELECT user_id FROM articles WHERE id = ?")
        .bind(article_id.as_str())
        .fetch_optional(pool.as_ref())
        .await?
        .ok_or(AppError::NotFound)?;

    let user_id: String = article.get("user_id");
    if user_id != claims.sub {
        return Err(AppError::Forbidden);
    }

    if let Some(title) = &input.title {
        sqlx::query("UPDATE articles SET title = ? WHERE id = ?")
            .bind(title)
            .bind(article_id.as_str())
            .execute(pool.as_ref())
            .await?;
    }

    if let Some(content) = &input.content {
        sqlx::query("UPDATE articles SET content = ? WHERE id = ?")
            .bind(content)
            .bind(article_id.as_str())
            .execute(pool.as_ref())
            .await?;
    }

    let updated_article = sqlx::query(
        r#"
        SELECT
            a.id, a.user_id, a.title, a.content,
            a.created_at, a.updated_at, u.username
        FROM articles a
        JOIN users u ON a.user_id = u.id
        WHERE a.id = ?
        "#
    )
    .bind(article_id.as_str())
    .fetch_one(pool.as_ref())
    .await?;

    let response = ArticleResponse {
        id: updated_article.get("id"),
        user_id: updated_article.get("user_id"),
        username: updated_article.get("username"),
        title: updated_article.get("title"),
        content: updated_article.get("content"),
        created_at: updated_article.get("created_at"),
        updated_at: updated_article.get("updated_at"),
    };

    Ok(HttpResponse::Ok().json(response))
}

pub async fn delete_article(
    pool: web::Data<Pool<MySql>>,
    article_id: web::Path<String>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let extensions = req.extensions();
    let claims = extensions
        .get::<Claims>()
        .ok_or(AppError::AuthenticationError)?
        .clone();

    let article = sqlx::query("SELECT user_id FROM articles WHERE id = ?")
        .bind(article_id.as_str())
        .fetch_optional(pool.as_ref())
        .await?
        .ok_or(AppError::NotFound)?;

    let user_id: String = article.get("user_id");
    if user_id != claims.sub {
        return Err(AppError::Forbidden);
    }

    sqlx::query("DELETE FROM articles WHERE id = ?")
        .bind(article_id.as_str())
        .execute(pool.as_ref())
        .await?;

    Ok(HttpResponse::NoContent().finish())
}