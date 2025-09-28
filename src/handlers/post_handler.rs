use crate::db::DbPool;
use crate::models::{Claims, CreatePostRequest, Post, PostResponse, UpdatePostRequest};
use actix_web::{web, HttpResponse, Result};

pub async fn create_post(
    pool: web::Data<DbPool>,
    req: web::Json<CreatePostRequest>,
    claims: web::ReqData<Claims>,
) -> Result<HttpResponse> {
    let user_id = claims.into_inner().sub;
    let result = sqlx::query(
        "INSERT INTO posts (user_id, title, content) VALUES (?, ?, ?)"
    )
    .bind(user_id)
    .bind(&req.title)
    .bind(&req.content)
    .execute(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    let post_id = result.last_insert_id() as i32;

    let post = sqlx::query_as::<_, PostResponse>(
        "SELECT p.id, p.user_id, u.username, p.title, p.content, p.created_at, p.updated_at
         FROM posts p
         JOIN users u ON p.user_id = u.id
         WHERE p.id = ?"
    )
    .bind(post_id)
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    Ok(HttpResponse::Created().json(post))
}

pub async fn get_posts(
    pool: web::Data<DbPool>,
) -> Result<HttpResponse> {
    let posts = sqlx::query_as::<_, PostResponse>(
        "SELECT p.id, p.user_id, u.username, p.title, p.content, p.created_at, p.updated_at
         FROM posts p
         JOIN users u ON p.user_id = u.id
         ORDER BY p.created_at DESC"
    )
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    Ok(HttpResponse::Ok().json(posts))
}

pub async fn get_post(
    pool: web::Data<DbPool>,
    post_id: web::Path<i32>,
) -> Result<HttpResponse> {
    let post = sqlx::query_as::<_, PostResponse>(
        "SELECT p.id, p.user_id, u.username, p.title, p.content, p.created_at, p.updated_at
         FROM posts p
         JOIN users u ON p.user_id = u.id
         WHERE p.id = ?"
    )
    .bind(post_id.into_inner())
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    match post {
        Some(post) => Ok(HttpResponse::Ok().json(post)),
        None => Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "Post not found"
        }))),
    }
}

pub async fn update_post(
    pool: web::Data<DbPool>,
    post_id: web::Path<i32>,
    req: web::Json<UpdatePostRequest>,
    claims: web::ReqData<Claims>,
) -> Result<HttpResponse> {
    let post = sqlx::query_as::<_, Post>(
        "SELECT * FROM posts WHERE id = ?"
    )
    .bind(post_id.into_inner())
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    match post {
        Some(post) => {
            if post.user_id != claims.into_inner().sub {
                return Ok(HttpResponse::Forbidden().json(serde_json::json!({
                    "error": "You don't have permission to update this post"
                })));
            }

            let title = req.title.as_ref().unwrap_or(&post.title);
            let content = req.content.as_ref().unwrap_or(&post.content);

            sqlx::query(
                "UPDATE posts SET title = ?, content = ? WHERE id = ?"
            )
            .bind(title)
            .bind(content)
            .bind(post.id)
            .execute(pool.get_ref())
            .await
            .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

            let updated_post = sqlx::query_as::<_, PostResponse>(
                "SELECT p.id, p.user_id, u.username, p.title, p.content, p.created_at, p.updated_at
                 FROM posts p
                 JOIN users u ON p.user_id = u.id
                 WHERE p.id = ?"
            )
            .bind(post.id)
            .fetch_one(pool.get_ref())
            .await
            .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

            Ok(HttpResponse::Ok().json(updated_post))
        }
        None => Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "Post not found"
        }))),
    }
}

pub async fn delete_post(
    pool: web::Data<DbPool>,
    post_id: web::Path<i32>,
    claims: web::ReqData<Claims>,
) -> Result<HttpResponse> {
    let post = sqlx::query_as::<_, Post>(
        "SELECT * FROM posts WHERE id = ?"
    )
    .bind(post_id.into_inner())
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    match post {
        Some(post) => {
            if post.user_id != claims.into_inner().sub {
                return Ok(HttpResponse::Forbidden().json(serde_json::json!({
                    "error": "You don't have permission to delete this post"
                })));
            }

            sqlx::query("DELETE FROM posts WHERE id = ?")
                .bind(post.id)
                .execute(pool.get_ref())
                .await
                .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

            Ok(HttpResponse::Ok().json(serde_json::json!({
                "message": "Post deleted successfully"
            })))
        }
        None => Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "Post not found"
        }))),
    }
}