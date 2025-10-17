use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;
use thiserror::Error;

/// 导演模块错误类型
#[derive(Debug, Error)]
#[allow(dead_code)]
pub enum DirectorError {
    #[error("Game not found")]
    GameNotFound,

    #[error("Invalid director password")]
    InvalidDirectorPassword,

    #[error("Player name already exists: {name}")]
    PlayerNameExists { name: String },

    #[error("Invalid password format")]
    InvalidPasswordFormat,

    #[error("Player not found: {id}")]
    PlayerNotFound { id: String },

    #[error("Cannot delete players after game started")]
    GameAlreadyStarted,

    #[error("Invalid player name: {reason}")]
    InvalidPlayerName { reason: String },

    #[error("Invalid team_id: {team_id}")]
    InvalidTeamId { team_id: i32 },

    #[error("Game_id mismatch: expected {expected}, got {actual}")]
    GameIdMismatch { expected: String, actual: String },

    #[error("Invalid game state transition")]
    InvalidGameStateTransition,

    #[error("Other error: {message}")]
    OtherError { message: String },

    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("UUID generation error: {0}")]
    UuidError(#[from] uuid::Error),

    #[error("Validation error: {message}")]
    ValidationError { message: String },
}

impl IntoResponse for DirectorError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            DirectorError::GameNotFound => (StatusCode::NOT_FOUND, self.to_string()),
            DirectorError::InvalidDirectorPassword => (StatusCode::UNAUTHORIZED, self.to_string()),
            DirectorError::PlayerNameExists { .. } => (StatusCode::BAD_REQUEST, self.to_string()),
            DirectorError::InvalidPasswordFormat => (StatusCode::BAD_REQUEST, self.to_string()),
            DirectorError::PlayerNotFound { .. } => (StatusCode::NOT_FOUND, self.to_string()),
            DirectorError::GameAlreadyStarted => (StatusCode::CONFLICT, self.to_string()),
            DirectorError::InvalidPlayerName { .. } => (StatusCode::BAD_REQUEST, self.to_string()),
            DirectorError::InvalidTeamId { .. } => (StatusCode::BAD_REQUEST, self.to_string()),
            DirectorError::GameIdMismatch { .. } => (StatusCode::BAD_REQUEST, self.to_string()),
            DirectorError::InvalidGameStateTransition => {
                (StatusCode::BAD_REQUEST, self.to_string())
            }
            DirectorError::OtherError { .. } => {
                (StatusCode::INTERNAL_SERVER_ERROR, self.to_string())
            }
            DirectorError::ValidationError { .. } => (StatusCode::BAD_REQUEST, self.to_string()),
            DirectorError::DatabaseError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database error occurred".to_string(),
            ),
            DirectorError::UuidError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "UUID generation error".to_string(),
            ),
        };

        let body = Json(json!({
            "success": false,
            "error": {
                "message": error_message,
                "details": self.to_string()
            }
        }));

        (status, body).into_response()
    }
}

impl From<String> for DirectorError {
    fn from(message: String) -> Self {
        DirectorError::ValidationError { message }
    }
}

impl From<&str> for DirectorError {
    fn from(message: &str) -> Self {
        DirectorError::ValidationError {
            message: message.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let error = DirectorError::GameNotFound;
        assert_eq!(error.to_string(), "Game not found");

        let error = DirectorError::PlayerNameExists {
            name: "test".to_string(),
        };
        assert_eq!(error.to_string(), "Player name already exists: test");

        let error = DirectorError::ValidationError {
            message: "test validation".to_string(),
        };
        assert_eq!(error.to_string(), "Validation error: test validation");
    }

    #[test]
    fn test_error_conversion_from_string() {
        let error: DirectorError = "test error".into();
        match error {
            DirectorError::ValidationError { message } => {
                assert_eq!(message, "test error");
            }
            _ => panic!("Expected ValidationError"),
        }
    }
}
