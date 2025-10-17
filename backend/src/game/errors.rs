use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;
use thiserror::Error;

/// 游戏相关错误
#[derive(Debug, Error)]
#[allow(dead_code)]
pub enum GameError {
    #[error("游戏不存在")]
    GameNotFound,

    #[error("规则模板不存在")]
    RuleTemplateNotFound,

    #[error("游戏名称已存在")]
    GameNameExists,

    #[error("游戏状态不允许此操作")]
    InvalidGameState,

    #[error("验证失败: {0}")]
    ValidationError(String),

    #[error("数据库错误: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("其他错误: {0}")]
    OtherError(String),
}

impl From<String> for GameError {
    fn from(error: String) -> Self {
        GameError::OtherError(error)
    }
}

impl IntoResponse for GameError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            GameError::GameNotFound => (StatusCode::NOT_FOUND, "游戏不存在"),
            GameError::RuleTemplateNotFound => (StatusCode::BAD_REQUEST, "规则模板不存在"),
            GameError::GameNameExists => (StatusCode::CONFLICT, "游戏名称已存在"),
            GameError::InvalidGameState => (StatusCode::BAD_REQUEST, "游戏状态不允许此操作"),
            GameError::ValidationError(ref msg) => (StatusCode::BAD_REQUEST, msg.as_str()),
            GameError::DatabaseError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "数据库操作失败"),
            GameError::OtherError(ref msg) => (StatusCode::BAD_REQUEST, msg.as_str()),
        };

        let body = Json(json!({
            "success": false,
            "error": message
        }));

        (status, body).into_response()
    }
}
