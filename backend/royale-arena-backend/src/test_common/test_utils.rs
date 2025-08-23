// Common test utilities for the Royale Arena backend

use crate::AppState;
use actix_web::{App, web};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Creates a test application instance with the provided app state
pub fn create_test_app(
    app_state: Arc<Mutex<AppState>>,
) -> App<
    impl actix_web::dev::ServiceFactory<
        actix_web::dev::ServiceRequest,
        Config = (),
        Response = actix_web::dev::ServiceResponse<actix_web::body::BoxBody>,
        Error = actix_web::Error,
        InitError = (),
    >,
> {
    App::new().app_data(web::Data::new(app_state.clone()))
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