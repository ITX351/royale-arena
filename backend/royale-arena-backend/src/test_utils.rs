// Common test utilities for the Royale Arena backend

use actix_web::{web, App};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::{AppState, models::game::Game};

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
    }))
}