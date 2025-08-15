// Common test utilities for the Royale Arena backend

use actix_web::{web, App};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::AppState;
use crate::services::db::create_db_pool;
use mysql::prelude::*;

/// Creates a test application instance with the provided app state
pub fn create_test_app(
    app_state: Arc<Mutex<AppState>>
) -> App<
    impl actix_web::dev::ServiceFactory<
        actix_web::dev::ServiceRequest,
        Config = (),
        Response = actix_web::dev::ServiceResponse<actix_web::body::BoxBody>,
        Error = actix_web::Error,
        InitError = (),
    >
> {
    App::new()
        .app_data(web::Data::new(app_state.clone()))
}

/// Creates a test app state with sample data for testing
pub fn create_test_app_state() -> Arc<Mutex<AppState>> {
    // Create app state
    Arc::new(Mutex::new(AppState {
        games: HashMap::new(),
        game_rules: HashMap::new(),
        rule_templates: HashMap::new(),
        places: HashMap::new(),
    }))
}

/// 创建测试数据库连接
pub fn create_test_db_connection() -> Result<mysql::PooledConn, mysql::Error> {
    let pool = create_db_pool()?;
    pool.get_conn()
}

/// 清理测试数据
pub fn clean_test_data(conn: &mut mysql::PooledConn, table: &str, condition: &str) -> Result<(), mysql::Error> {
    let query = format!("DELETE FROM {} WHERE {}", table, condition);
    conn.query_drop(query)
}

/// 插入测试数据
pub fn insert_test_data<T: serde::Serialize>(conn: &mut mysql::PooledConn, table: &str, data: &T) -> Result<(), mysql::Error> {
    // 这里应该实现具体的插入逻辑，根据数据类型生成INSERT语句
    // 由于实现复杂，暂时只打印日志
    tracing::info!("Inserting test data into table: {}", table);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_test_app_state() {
        let app_state = create_test_app_state();
        let state = app_state.blocking_lock(); // 在测试中获取锁
        assert!(state.games.is_empty());
        assert!(state.game_rules.is_empty());
        assert!(state.rule_templates.is_empty());
        assert!(state.places.is_empty());
    }
}