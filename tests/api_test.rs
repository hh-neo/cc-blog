#[cfg(test)]
mod tests {
    use serde_json::json;

    #[test]
    fn test_user_registration_payload() {
        let payload = json!({
            "username": "testuser",
            "email": "test@example.com",
            "password": "testpass123"
        });

        assert_eq!(payload["username"], "testuser");
        assert_eq!(payload["email"], "test@example.com");
    }

    #[test]
    fn test_article_creation_payload() {
        let payload = json!({
            "title": "Test Article",
            "content": "This is test content"
        });

        assert_eq!(payload["title"], "Test Article");
        assert_eq!(payload["content"], "This is test content");
    }

    #[test]
    fn test_login_payload() {
        let payload = json!({
            "username": "testuser",
            "password": "testpass123"
        });

        assert_eq!(payload["username"], "testuser");
        assert_eq!(payload["password"], "testpass123");
    }
}