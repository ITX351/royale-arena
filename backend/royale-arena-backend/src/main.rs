use actix_web::{web, App, HttpServer, middleware::Logger};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing_subscriber;

// Import modules
mod models;
mod handlers;
mod routes;
mod services;
#[cfg(test)]
mod test_utils;

use models::game::Game;
use models::admin::AdminUser;

// AppState structure
pub struct AppState {
    pub games: HashMap<String, Game>,
    pub admin_users: HashMap<String, AdminUser>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    // Initialize in-memory state with some test data
    let mut admin_users = HashMap::new();
    admin_users.insert(
        "admin".to_string(),
        AdminUser {
            username: "admin".to_string(),
            password: "password123".to_string(), // In production, this should be hashed
        },
    );
    
    let app_state = Arc::new(Mutex::new(AppState {
        games: HashMap::new(),
        admin_users,
    }));
    
    // Start HTTP server
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .wrap(Logger::default())
            .configure(routes::configure_routes)
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}