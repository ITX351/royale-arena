use actix_web::{web, HttpResponse, Result};
use serde_json::json;
use crate::models::game::Game;

pub async fn get_games(data: web::Data<std::sync::Arc<tokio::sync::Mutex<crate::AppState>>>) -> Result<HttpResponse> {
    let state = data.lock().await;
    let games: Vec<&Game> = state.games.values().collect();
    
    Ok(HttpResponse::Ok().json(json!({
        "games": games
    })))
}

pub async fn get_game_info(path: web::Path<String>, data: web::Data<std::sync::Arc<tokio::sync::Mutex<crate::AppState>>>) -> Result<HttpResponse> {
    let game_id = path.into_inner();
    let state = data.lock().await;
    
    match state.games.get(&game_id) {
        Some(game) => Ok(HttpResponse::Ok().json(game)),
        None => Ok(HttpResponse::NotFound().json(json!({
            "error": "Game not found"
        })))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, web, App, http::StatusCode};
    use serde_json::Value;
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::Mutex;
    use crate::AppState;

    #[actix_web::test]
    async fn test_get_games_empty() {
        // Create test app state with no games
        let app_state = Arc::new(Mutex::new(AppState {
            games: HashMap::new(),
            admin_users: HashMap::new(),
        }));

        // Create test app
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(app_state.clone()))
                .route("/games", web::get().to(get_games))
        ).await;

        // Make request
        let req = test::TestRequest::get().uri("/games").to_request();
        let resp = test::call_service(&app, req).await;

        // Check response
        assert_eq!(resp.status(), StatusCode::OK);
        
        let body = test::read_body(resp).await;
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert!(json.get("games").is_some());
        assert_eq!(json["games"].as_array().unwrap().len(), 0);
    }

    #[actix_web::test]
    async fn test_get_game_info_not_found() {
        // Create test app state with no games
        let app_state = Arc::new(Mutex::new(AppState {
            games: HashMap::new(),
            admin_users: HashMap::new(),
        }));

        // Create test app
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(app_state.clone()))
                .route("/game/{game_id}", web::get().to(get_game_info))
        ).await;

        // Make request for non-existent game
        let req = test::TestRequest::get().uri("/game/test_game").to_request();
        let resp = test::call_service(&app, req).await;

        // Check response
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
        
        let body = test::read_body(resp).await;
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert!(json.get("error").is_some());
        assert_eq!(json["error"], "Game not found");
    }
}