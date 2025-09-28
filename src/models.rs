use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserResponse,
}

#[derive(Debug, Serialize, FromRow)]
pub struct UserResponse {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Post {
    pub id: i32,
    pub user_id: i32,
    pub title: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePostRequest {
    pub title: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePostRequest {
    pub title: Option<String>,
    pub content: Option<String>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct PostResponse {
    pub id: i32,
    pub user_id: i32,
    pub username: String,
    pub title: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i32,
    pub username: String,
    pub exp: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_request_deserialization() {
        let json = r#"{"username":"test","email":"test@test.com","password":"pass123"}"#;
        let req: Result<RegisterRequest, _> = serde_json::from_str(json);
        assert!(req.is_ok());
        let req = req.unwrap();
        assert_eq!(req.username, "test");
        assert_eq!(req.email, "test@test.com");
        assert_eq!(req.password, "pass123");
    }

    #[test]
    fn test_login_request_deserialization() {
        let json = r#"{"username":"test","password":"pass123"}"#;
        let req: Result<LoginRequest, _> = serde_json::from_str(json);
        assert!(req.is_ok());
        let req = req.unwrap();
        assert_eq!(req.username, "test");
        assert_eq!(req.password, "pass123");
    }

    #[test]
    fn test_create_post_request_deserialization() {
        let json = r#"{"title":"Test Title","content":"Test Content"}"#;
        let req: Result<CreatePostRequest, _> = serde_json::from_str(json);
        assert!(req.is_ok());
        let req = req.unwrap();
        assert_eq!(req.title, "Test Title");
        assert_eq!(req.content, "Test Content");
    }

    #[test]
    fn test_update_post_request_deserialization() {
        let json = r#"{"title":"Updated Title"}"#;
        let req: Result<UpdatePostRequest, _> = serde_json::from_str(json);
        assert!(req.is_ok());
        let req = req.unwrap();
        assert_eq!(req.title, Some("Updated Title".to_string()));
        assert_eq!(req.content, None);
    }
}