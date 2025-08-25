pub mod admin;
pub mod auth;
pub mod config;
pub mod database;
pub mod errors;
pub mod routes;
pub mod rule_template;

// 选择性导出，避免命名冲突
pub use config::AppConfig;
pub use database::{create_pool, DatabasePool};
pub use errors::{AppError, AuthError, ServiceError};
pub use rule_template::{RuleTemplateService, RuleTemplateError};