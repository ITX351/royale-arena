use thiserror::Error;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

#[derive(Debug, Error)]
#[allow(dead_code)]
pub enum RuleTemplateError {
    #[error("模版名称已存在")]
    NameAlreadyExists,
    
    #[error("模版不存在")]
    TemplateNotFound,
    
    #[error("无效的JSON配置: {0}")]
    InvalidRulesConfig(String),
    
    #[error("模版名称不能为空")]
    EmptyTemplateName,
    
    #[error("验证失败: {0}")]
    ValidationError(String),
    
    #[error("数据库错误: {0}")]
    DatabaseError(#[from] sqlx::Error),
    
    #[error("UUID解析错误: {0}")]
    UuidError(#[from] uuid::Error),
    
    #[error("JSON序列化错误: {0}")]
    JsonError(#[from] serde_json::Error),
}

impl IntoResponse for RuleTemplateError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            RuleTemplateError::NameAlreadyExists => {
                (StatusCode::CONFLICT, "模版名称已存在")
            }
            RuleTemplateError::TemplateNotFound => {
                (StatusCode::NOT_FOUND, "模版不存在")
            }
            RuleTemplateError::InvalidRulesConfig(_) => {
                (StatusCode::BAD_REQUEST, "无效的规则配置")
            }
            RuleTemplateError::EmptyTemplateName => {
                (StatusCode::BAD_REQUEST, "模版名称不能为空")
            }
            RuleTemplateError::ValidationError(_) => {
                (StatusCode::BAD_REQUEST, "参数验证失败")
            }
            RuleTemplateError::UuidError(_) => {
                (StatusCode::BAD_REQUEST, "无效的ID格式")
            }
            RuleTemplateError::JsonError(_) => {
                (StatusCode::BAD_REQUEST, "JSON格式错误")
            }
            RuleTemplateError::DatabaseError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "服务器内部错误")
            }
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