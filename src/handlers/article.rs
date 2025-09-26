use crate::middleware::AuthUser;
use crate::models::article::{
    Article, ArticleWithAuthor, Comment, CommentWithUser, CreateArticleInput,
    CreateCommentInput, PaginatedResponse, PaginationQuery, UpdateArticleInput,
};
use crate::utils::{AppError, AppResult};
use actix_web::{web, HttpResponse};
use sqlx::MySqlPool;
use validator::Validate;

pub async fn create_article(
    pool: web::Data<MySqlPool>,
    auth_user: AuthUser,
    input: web::Json<CreateArticleInput>,
) -> AppResult<HttpResponse> {
    input.validate().map_err(|e| AppError::ValidationError(e.to_string()))?;

    let article = Article::new(
        input.title.clone(),
        input.content.clone(),
        auth_user.user_id.clone(),
    );

    sqlx::query!(
        "INSERT INTO articles (id, title, content, author_id, view_count, created_at, updated_at, is_published)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        article.id,
        article.title,
        article.content,
        article.author_id,
        article.view_count,
        article.created_at,
        article.updated_at,
        input.is_published.unwrap_or(true)
    )
    .execute(pool.get_ref())
    .await?;

    Ok(HttpResponse::Created().json(article))
}

pub async fn get_articles(
    pool: web::Data<MySqlPool>,
    query: web::Query<PaginationQuery>,
) -> AppResult<HttpResponse> {
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(10).min(100).max(1);
    let offset = ((page - 1) * per_page) as i64;

    let total = sqlx::query_scalar!("SELECT COUNT(*) FROM articles WHERE is_published = true")
        .fetch_one(pool.get_ref())
        .await?;

    let articles = sqlx::query_as!(
        ArticleWithAuthor,
        "SELECT a.*, u.username as author_username
         FROM articles a
         JOIN users u ON a.author_id = u.id
         WHERE a.is_published = true
         ORDER BY a.created_at DESC
         LIMIT ? OFFSET ?",
        per_page as i64,
        offset
    )
    .fetch_all(pool.get_ref())
    .await?;

    let total_pages = ((total as f64) / (per_page as f64)).ceil() as u32;

    Ok(HttpResponse::Ok().json(PaginatedResponse {
        data: articles,
        total,
        page,
        per_page,
        total_pages,
    }))
}

pub async fn get_article(
    pool: web::Data<MySqlPool>,
    article_id: web::Path<String>,
) -> AppResult<HttpResponse> {
    let article = sqlx::query_as!(
        ArticleWithAuthor,
        "SELECT a.*, u.username as author_username
         FROM articles a
         JOIN users u ON a.author_id = u.id
         WHERE a.id = ? AND a.is_published = true",
        article_id.as_str()
    )
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or(AppError::NotFoundError)?;

    sqlx::query!(
        "UPDATE articles SET view_count = view_count + 1 WHERE id = ?",
        article_id.as_str()
    )
    .execute(pool.get_ref())
    .await?;

    Ok(HttpResponse::Ok().json(article))
}

pub async fn update_article(
    pool: web::Data<MySqlPool>,
    auth_user: AuthUser,
    article_id: web::Path<String>,
    input: web::Json<UpdateArticleInput>,
) -> AppResult<HttpResponse> {
    input.validate().map_err(|e| AppError::ValidationError(e.to_string()))?;

    let article = sqlx::query_as!(
        Article,
        "SELECT * FROM articles WHERE id = ?",
        article_id.as_str()
    )
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or(AppError::NotFoundError)?;

    if article.author_id != auth_user.user_id {
        return Err(AppError::UnauthorizedError);
    }

    let mut update_query = String::from("UPDATE articles SET updated_at = NOW()");
    let mut params = vec![];

    if let Some(title) = &input.title {
        update_query.push_str(", title = ?");
        params.push(title.clone());
    }

    if let Some(content) = &input.content {
        update_query.push_str(", content = ?");
        params.push(content.clone());
    }

    if let Some(is_published) = input.is_published {
        update_query.push_str(", is_published = ?");
        params.push(is_published.to_string());
    }

    update_query.push_str(" WHERE id = ?");

    let mut query = sqlx::query(&update_query);
    for param in params {
        query = query.bind(param);
    }
    query = query.bind(article_id.as_str());

    query.execute(pool.get_ref()).await?;

    let updated_article = sqlx::query_as!(
        Article,
        "SELECT * FROM articles WHERE id = ?",
        article_id.as_str()
    )
    .fetch_one(pool.get_ref())
    .await?;

    Ok(HttpResponse::Ok().json(updated_article))
}

pub async fn delete_article(
    pool: web::Data<MySqlPool>,
    auth_user: AuthUser,
    article_id: web::Path<String>,
) -> AppResult<HttpResponse> {
    let article = sqlx::query!(
        "SELECT author_id FROM articles WHERE id = ?",
        article_id.as_str()
    )
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or(AppError::NotFoundError)?;

    if article.author_id != auth_user.user_id {
        return Err(AppError::UnauthorizedError);
    }

    sqlx::query!("DELETE FROM articles WHERE id = ?", article_id.as_str())
        .execute(pool.get_ref())
        .await?;

    Ok(HttpResponse::NoContent().finish())
}

pub async fn create_comment(
    pool: web::Data<MySqlPool>,
    auth_user: AuthUser,
    article_id: web::Path<String>,
    input: web::Json<CreateCommentInput>,
) -> AppResult<HttpResponse> {
    input.validate().map_err(|e| AppError::ValidationError(e.to_string()))?;

    let article = sqlx::query!(
        "SELECT id FROM articles WHERE id = ? AND is_published = true",
        article_id.as_str()
    )
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or(AppError::NotFoundError)?;

    let comment = Comment::new(
        article_id.to_string(),
        auth_user.user_id.clone(),
        input.content.clone(),
    );

    sqlx::query!(
        "INSERT INTO comments (id, article_id, user_id, content, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?)",
        comment.id,
        comment.article_id,
        comment.user_id,
        comment.content,
        comment.created_at,
        comment.updated_at
    )
    .execute(pool.get_ref())
    .await?;

    Ok(HttpResponse::Created().json(comment))
}

pub async fn get_comments(
    pool: web::Data<MySqlPool>,
    article_id: web::Path<String>,
) -> AppResult<HttpResponse> {
    let comments = sqlx::query_as!(
        CommentWithUser,
        "SELECT c.*, u.username
         FROM comments c
         JOIN users u ON c.user_id = u.id
         WHERE c.article_id = ?
         ORDER BY c.created_at DESC",
        article_id.as_str()
    )
    .fetch_all(pool.get_ref())
    .await?;

    Ok(HttpResponse::Ok().json(comments))
}

pub async fn search_articles(
    pool: web::Data<MySqlPool>,
    query: web::Query<SearchQuery>,
) -> AppResult<HttpResponse> {
    let search_term = format!("%{}%", query.q);

    let articles = sqlx::query_as!(
        ArticleWithAuthor,
        "SELECT a.*, u.username as author_username
         FROM articles a
         JOIN users u ON a.author_id = u.id
         WHERE a.is_published = true
         AND (a.title LIKE ? OR a.content LIKE ?)
         ORDER BY a.created_at DESC
         LIMIT 20",
        search_term,
        search_term
    )
    .fetch_all(pool.get_ref())
    .await?;

    Ok(HttpResponse::Ok().json(articles))
}

#[derive(serde::Deserialize)]
pub struct SearchQuery {
    pub q: String,
}