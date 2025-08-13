use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct AdminUser {
    pub username: String,
    pub password: String, // In production, this should be hashed
}

#[derive(Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Deserialize)]
pub struct LoginResponse {
    pub success: bool,
    pub token: Option<String>,
    pub expires_in: Option<i64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_admin_user_creation() {
        let admin = AdminUser {
            username: "admin".to_string(),
            password: "password123".to_string(),
        };

        assert_eq!(admin.username, "admin");
        assert_eq!(admin.password, "password123");
    }

    #[test]
    fn test_login_request_creation() {
        let request = LoginRequest {
            username: "admin".to_string(),
            password: "password123".to_string(),
        };

        assert_eq!(request.username, "admin");
        assert_eq!(request.password, "password123");
    }

    #[test]
    fn test_login_response_creation_success() {
        let response = LoginResponse {
            success: true,
            token: Some("token123".to_string()),
            expires_in: Some(3600),
        };

        assert!(response.success);
        assert_eq!(response.token, Some("token123".to_string()));
        assert_eq!(response.expires_in, Some(3600));
    }

    #[test]
    fn test_login_response_creation_failure() {
        let response = LoginResponse {
            success: false,
            token: None,
            expires_in: None,
        };

        assert!(!response.success);
        assert_eq!(response.token, None);
        assert_eq!(response.expires_in, None);
    }

    #[test]
    fn test_admin_user_serialization() {
        let admin = AdminUser {
            username: "admin".to_string(),
            password: "password123".to_string(),
        };

        let serialized = serde_json::to_string(&admin).unwrap();
        let deserialized: AdminUser = serde_json::from_str(&serialized).unwrap();

        assert_eq!(admin.username, deserialized.username);
        assert_eq!(admin.password, deserialized.password);
    }

    #[test]
    fn test_login_request_deserialization() {
        let json = r#"{"username": "admin", "password": "password123"}"#;
        let request: LoginRequest = serde_json::from_str(json).unwrap();

        assert_eq!(request.username, "admin");
        assert_eq!(request.password, "password123");
    }

    #[test]
    fn test_login_response_serialization() {
        let response = LoginResponse {
            success: true,
            token: Some("token123".to_string()),
            expires_in: Some(3600),
        };

        let serialized = serde_json::to_string(&response).unwrap();
        let deserialized: LoginResponse = serde_json::from_str(&serialized).unwrap();

        assert_eq!(response.success, deserialized.success);
        assert_eq!(response.token, deserialized.token);
        assert_eq!(response.expires_in, deserialized.expires_in);
    }
}