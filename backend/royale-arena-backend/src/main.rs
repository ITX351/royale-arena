use actix_web::{web, App, HttpServer, HttpResponse, Result, middleware::Logger};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// Data models
#[derive(Serialize, Deserialize, Clone)]
struct Game {
    id: String,
    name: String,
    description: String,
    status: String, // waiting|running|paused|ended
    phase: String,  // day|night
    player_count: u32,
    max_players: u32,
    start_time: Option<String>,
    end_time: Option<String>,
    action_start_time: Option<String>,
    action_end_time: Option<String>,
    safe_zones: Vec<String>,
    weather: f64,
    announcements: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone)]
struct Player {
    id: String,
    name: String,
    password: String, // 6-8位字母数字
    life: u32,        // 生命值 (0-100)
    strength: u32,    // 体力值 (0-100)
    location: String, // 当前位置
    things: Vec<String>, // 拥有的道具列表
    hands: Vec<String>,  // 装备的道具列表
    able: bool,       // 是否可行动
    injured: u32,     // 是否受伤 (持续伤害标记)
    vote: u32,        // 持有的票数
    ts: u64,          // 上次搜索时间戳
    deliver: u32,     // 传音次数标记
    rest: u32,        // 静养模式标记
}

// Simple in-memory storage
struct AppState {
    games: HashMap<String, Game>,
    // players: HashMap<String, Player>, // Will be used later
}

// API handlers
async fn get_games(data: web::Data<Arc<Mutex<AppState>>>) -> Result<HttpResponse> {
    let state = data.lock().unwrap();
    let games: Vec<&Game> = state.games.values().collect();
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "games": games
    })))
}

async fn get_game_info(path: web::Path<String>, data: web::Data<Arc<Mutex<AppState>>>) -> Result<HttpResponse> {
    let game_id = path.into_inner();
    let state = data.lock().unwrap();
    
    match state.games.get(&game_id) {
        Some(game) => Ok(HttpResponse::Ok().json(game)),
        None => Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "Game not found"
        })))
    }
}

// Main function
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    // Initialize in-memory state
    let app_state = Arc::new(Mutex::new(AppState {
        games: HashMap::new(),
        // players: HashMap::new(),
    }));
    
    // Start HTTP server
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .wrap(Logger::default())
            .route("/api/games", web::get().to(get_games))
            .route("/api/game/{game_id}", web::get().to(get_game_info))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}