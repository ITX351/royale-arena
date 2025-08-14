use actix_web::{web, HttpResponse, Result};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct WsAuthRequest {
    pub game_id: String,
    pub password: String, // 可以是导演密码或演员密码
    pub user_type: String, // "director" or "player"
    pub player_id: Option<String>, // 如果是玩家，需要提供player_id
}

#[derive(Serialize, Deserialize)]
pub struct WsAuthResponse {
    pub success: bool,
    pub message: String,
    pub token: Option<String>,
}

// 验证WebSocket连接凭据
pub async fn ws_auth(
    _req: web::Json<WsAuthRequest>,
    _data: web::Data<std::sync::Arc<tokio::sync::Mutex<crate::AppState>>>
) -> Result<HttpResponse> {
    // 在实际实现中，我们需要：
    // 1. 验证游戏ID和密码
    // 2. 根据用户类型验证权限
    // 3. 生成临时访问令牌
    
    // 目前返回示例响应
    let response = WsAuthResponse {
        success: true,
        message: "Authentication successful".to_string(),
        token: Some("ws-token-12345".to_string()),
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
    async fn test_ws_auth_director() {
        // Create test app state and app
        let app_state = create_test_app_state();
        let app = test::init_service(
            create_test_app(app_state.clone())
                .route("/game/{game_id}/ws-auth", web::post().to(ws_auth))
        ).await;

        // Create request data for director
        let request_data = WsAuthRequest {
            game_id: "test_game".to_string(),
            password: "director123".to_string(),
            user_type: "director".to_string(),
            player_id: None,
        };

        // Make request
        let req = test::TestRequest::post()
            .uri("/game/test_game/ws-auth")
            .set_json(&request_data)
            .to_request();
        let resp = test::call_service(&app, req).await;

        // Check response
        assert!(resp.status().is_success());
        
        let body = test::read_body(resp).await;
        let json: Value = serde_json::from_slice(&body).unwrap();
        
        assert_eq!(json["success"], true);
        assert_eq!(json["message"], "Authentication successful");
        assert!(json.get("token").is_some());
    }

    #[actix_web::test]
    async fn test_ws_auth_player() {
        // Create test app state and app
        let app_state = create_test_app_state();
        let app = test::init_service(
            create_test_app(app_state.clone())
                .route("/game/{game_id}/ws-auth", web::post().to(ws_auth))
        ).await;

        // Create request data for player
        let request_data = WsAuthRequest {
            game_id: "test_game".to_string(),
            password: "player123".to_string(),
            user_type: "player".to_string(),
            player_id: Some("player-1".to_string()),
        };

        // Make request
        let req = test::TestRequest::post()
            .uri("/game/test_game/ws-auth")
            .set_json(&request_data)
            .to_request();
        let resp = test::call_service(&app, req).await;

        // Check response
        assert!(resp.status().is_success());
        
        let body = test::read_body(resp).await;
        let json: Value = serde_json::from_slice(&body).unwrap();
        
        assert_eq!(json["success"], true);
        assert_eq!(json["message"], "Authentication successful");
        assert!(json.get("token").is_some());
    }
}