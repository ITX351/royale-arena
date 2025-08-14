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

// AppState structure
pub struct AppState {
    pub games: HashMap<String, Game>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables from .env.royale file
    match dotenvy::from_filename(".env.royale") {
        Ok(_) => println!("Successfully loaded .env.royale file"),
        Err(e) => eprintln!("Warning: Failed to load .env.royale file: {}", e),
    }
    
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    let app_state = Arc::new(Mutex::new(AppState {
        games: HashMap::new(),
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