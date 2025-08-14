use actix_web::{web, HttpResponse, Result};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct AddPlayersRequest {
    pub players: Vec<NewPlayer>,
}

#[derive(Serialize, Deserialize)]
pub struct NewPlayer {
    pub name: String,
    pub password: String,
    pub team_id: Option<u32>,
}

#[derive(Serialize, Deserialize)]
pub struct AddPlayersResponse {
    pub success: bool,
    pub message: String,
    pub added_players: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct PlayerListResponse {
    pub players: Vec<PlayerInfo>,
}

#[derive(Serialize, Deserialize)]
pub struct PlayerInfo {
    pub id: String,
    pub name: String,
    pub team_id: u32,
}

#[derive(Serialize, Deserialize)]
pub struct DeletePlayersRequest {
    pub player_ids: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct DeletePlayersResponse {
    pub success: bool,
    pub message: String,
    pub deleted_players: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct UpdateGameRulesRequest {
    pub day_duration: Option<u32>,
    pub night_duration: Option<u32>,
    pub max_life: Option<u32>,
    pub max_strength: Option<u32>,
    pub move_cost: Option<u32>,
    pub search_cost: Option<u32>,
}

#[derive(Serialize, Deserialize)]
pub struct UpdateGameRulesResponse {
    pub success: bool,
    pub message: String,
}

// 批量添加演员账户
pub async fn add_players(
    path: web::Path<String>, // game_id
    _req: web::Json<AddPlayersRequest>,
    _data: web::Data<std::sync::Arc<tokio::sync::Mutex<crate::AppState>>>
) -> Result<HttpResponse> {
    let game_id = path.into_inner();
    
    // 在实际实现中，我们需要：
    // 1. 验证导演权限
    // 2. 验证游戏是否存在
    // 3. 将新玩家添加到数据库
    // 4. 更新内存中的游戏状态
    
    // 目前返回一个示例响应
    let response = AddPlayersResponse {
        success: true,
        message: "Players added successfully".to_string(),
        added_players: vec![format!("player-{}-1", game_id), format!("player-{}-2", game_id)],
    };
    
    Ok(HttpResponse::Ok().json(response))
}

// 获取演员列表
pub async fn get_players(
    path: web::Path<String>, // game_id
    _data: web::Data<std::sync::Arc<tokio::sync::Mutex<crate::AppState>>>
) -> Result<HttpResponse> {
    let game_id = path.into_inner();
    
    // 在实际实现中，我们需要：
    // 1. 验证导演权限
    // 2. 从数据库获取该游戏的所有演员
    
    // 目前返回示例数据
    let players = vec![
        PlayerInfo {
            id: format!("player-{}-1", game_id),
            name: "演员1".to_string(),
            team_id: 0,
        },
        PlayerInfo {
            id: format!("player-{}-2", game_id),
            name: "演员2".to_string(),
            team_id: 1,
        }
    ];
    
    let response = PlayerListResponse { players };
    
    Ok(HttpResponse::Ok().json(response))
}

// 批量删除演员账户
pub async fn delete_players(
    path: web::Path<String>, // game_id
    _req: web::Json<DeletePlayersRequest>,
    _data: web::Data<std::sync::Arc<tokio::sync::Mutex<crate::AppState>>>
) -> Result<HttpResponse> {
    let game_id = path.into_inner();
    
    // 在实际实现中，我们需要：
    // 1. 验证导演权限
    // 2. 验证游戏是否存在
    // 3. 从数据库删除指定的演员
    
    // 目前返回一个示例响应
    let response = DeletePlayersResponse {
        success: true,
        message: "Players deleted successfully".to_string(),
        deleted_players: vec![format!("player-{}-1", game_id)],
    };
    
    Ok(HttpResponse::Ok().json(response))
}

// 更新游戏规则配置
pub async fn update_game_rules(
    path: web::Path<String>, // game_id
    _req: web::Json<UpdateGameRulesRequest>,
    _data: web::Data<std::sync::Arc<tokio::sync::Mutex<crate::AppState>>>
) -> Result<HttpResponse> {
    let _game_id = path.into_inner();
    
    // 在实际实现中，我们需要：
    // 1. 验证导演权限
    // 2. 验证游戏是否存在
    // 3. 更新游戏规则配置
    
    // 目前返回一个示例响应
    let response = UpdateGameRulesResponse {
        success: true,
        message: "Game rules updated successfully".to_string(),
    };
    
    Ok(HttpResponse::Ok().json(response))
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, web};
    use serde_json::Value;
    use crate::test_utils::{create_test_app, create_test_app_state};

    #[actix_web::test]
    async fn test_add_players() {
        // Create test app state and app
        let app_state = create_test_app_state();
        let app = test::init_service(
            create_test_app(app_state.clone())
                .route("/game/{game_id}/players", web::post().to(add_players))
        ).await;

        // Create request data
        let request_data = AddPlayersRequest {
            players: vec![
                NewPlayer {
                    name: "Test Player 1".to_string(),
                    password: "test123".to_string(),
                    team_id: Some(1),
                },
                NewPlayer {
                    name: "Test Player 2".to_string(),
                    password: "test456".to_string(),
                    team_id: None,
                }
            ]
        };

        // Make request
        let req = test::TestRequest::post()
            .uri("/game/test_game/players")
            .set_json(&request_data)
            .to_request();
        let resp = test::call_service(&app, req).await;

        // Check response
        assert!(resp.status().is_success());
        
        let body = test::read_body(resp).await;
        let json: Value = serde_json::from_slice(&body).unwrap();
        
        assert_eq!(json["success"], true);
        assert!(json.get("added_players").is_some());
    }

    #[actix_web::test]
    async fn test_get_players() {
        // Create test app state and app
        let app_state = create_test_app_state();
        let app = test::init_service(
            create_test_app(app_state.clone())
                .route("/game/{game_id}/players", web::get().to(get_players))
        ).await;

        // Make request
        let req = test::TestRequest::get().uri("/game/test_game/players").to_request();
        let resp = test::call_service(&app, req).await;

        // Check response
        assert!(resp.status().is_success());
        
        let body = test::read_body(resp).await;
        let json: Value = serde_json::from_slice(&body).unwrap();
        
        assert!(json.get("players").is_some());
    }

    #[actix_web::test]
    async fn test_delete_players() {
        // Create test app state and app
        let app_state = create_test_app_state();
        let app = test::init_service(
            create_test_app(app_state.clone())
                .route("/game/{game_id}/players", web::delete().to(delete_players))
        ).await;

        // Create request data
        let request_data = DeletePlayersRequest {
            player_ids: vec!["player-1".to_string(), "player-2".to_string()],
        };

        // Make request
        let req = test::TestRequest::delete()
            .uri("/game/test_game/players")
            .set_json(&request_data)
            .to_request();
        let resp = test::call_service(&app, req).await;

        // Check response
        assert!(resp.status().is_success());
        
        let body = test::read_body(resp).await;
        let json: Value = serde_json::from_slice(&body).unwrap();
        
        assert_eq!(json["success"], true);
        assert!(json.get("deleted_players").is_some());
    }

    #[actix_web::test]
    async fn test_update_game_rules() {
        // Create test app state and app
        let app_state = create_test_app_state();
        let app = test::init_service(
            create_test_app(app_state.clone())
                .route("/game/{game_id}/rules", web::put().to(update_game_rules))
        ).await;

        // Create request data
        let request_data = UpdateGameRulesRequest {
            day_duration: Some(600),
            night_duration: Some(300),
            max_life: Some(100),
            max_strength: Some(100),
            move_cost: Some(10),
            search_cost: Some(20),
        };

        // Make request
        let req = test::TestRequest::put()
            .uri("/game/test_game/rules")
            .set_json(&request_data)
            .to_request();
        let resp = test::call_service(&app, req).await;

        // Check response
        assert!(resp.status().is_success());
        
        let body = test::read_body(resp).await;
        let json: Value = serde_json::from_slice(&body).unwrap();
        
        assert_eq!(json["success"], true);
        assert_eq!(json["message"], "Game rules updated successfully");
    }
}