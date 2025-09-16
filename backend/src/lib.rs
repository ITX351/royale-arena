pub mod admin;
pub mod auth;
pub mod config;
pub mod database;
pub mod director;
pub mod errors;
pub mod game;
pub mod routes;
pub mod rule_template;
pub mod websocket;

// 选择性导出，避免命名冲突
pub use config::AppConfig;
pub use database::{create_pool, DatabasePool};
pub use director::{DirectorService, DirectorError};
pub use errors::{AppError, AuthError, ServiceError};
pub use game::{GameService, GameError};
pub use rule_template::{RuleTemplateService, RuleTemplateError};
pub use websocket::service::WebSocketService;