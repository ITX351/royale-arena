use actix_web::{test, web};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

// Import the modules from the main application
use royale_arena_backend::{
    AppState,
    handlers::rules,
    handlers::player,
};

#[actix_web::test]
async fn test_game_rules_handler() {
    // 创建应用状态
    let app_state = Arc::new(Mutex::new(AppState {
        games: HashMap::new(),
        game_rules: HashMap::new(),
    }));

    // 创建测试数据
    let path = web::Path::from("test_game".to_string());
    let data = web::Data::new(app_state.clone());

    // 调用handler函数
    let resp = rules::get_game_rules(path, data).await.unwrap();
    
    // 验证响应状态
    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn test_player_info_handler() {
    // 创建应用状态
    let app_state = Arc::new(Mutex::new(AppState {
        games: HashMap::new(),
        game_rules: HashMap::new(),
    }));

    // 创建测试数据
    let path = web::Path::from(("test_game".to_string(), "test_player".to_string()));
    let data = web::Data::new(app_state.clone());

    // 调用handler函数
    let resp = player::get_player_info(path, data).await.unwrap();
    
    // 验证响应状态
    assert!(resp.status().is_success());
}