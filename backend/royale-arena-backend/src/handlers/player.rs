use actix_web::{web, HttpResponse, Result};
use crate::models::player::Player;

pub async fn get_player_info(
    path: web::Path<(String, String)>, // (game_id, player_id)
    _data: web::Data<std::sync::Arc<tokio::sync::Mutex<crate::AppState>>>
) -> Result<HttpResponse> {
    let (_game_id, player_id) = path.into_inner();
    
    // 在实际实现中，我们需要从游戏数据中查找玩家信息
    // 这里我们暂时返回一个示例玩家信息
    // TODO: 实现从游戏数据中获取真实玩家信息的逻辑
    
    // 为了测试目的，我们创建一个示例玩家
    let player = Player {
        id: player_id.clone(),
        name: "Test Player".to_string(),
        password: "password123".to_string(),
        life: 100,
        strength: 100,
        location: "起始位置".to_string(),
        things: vec!["道具1".to_string(), "道具2".to_string()],
        hands: vec!["武器1".to_string()],
        able: true,
        injured: 0,
        vote: 1,
        ts: 1234567890,
        deliver: 0,
        rest: 0,
    };
    
    Ok(HttpResponse::Ok().json(player))
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, web};
    use serde_json::Value;
    use crate::test_utils::{create_test_app, create_test_app_state};

    #[actix_web::test]
    async fn test_get_player_info() {
        // Create test app state and app
        let app_state = create_test_app_state();
        let app = test::init_service(
            create_test_app(app_state.clone())
                .route("/game/{game_id}/player/{player_id}", web::get().to(get_player_info))
        ).await;

        // Make request for player info
        let req = test::TestRequest::get().uri("/game/test_game/player/test_player").to_request();
        let resp = test::call_service(&app, req).await;

        // Check response
        assert!(resp.status().is_success());
        
        let body = test::read_body(resp).await;
        let json: Value = serde_json::from_slice(&body).unwrap();
        
        // Check that we got a valid player object
        assert_eq!(json["id"], "test_player");
        assert_eq!(json["name"], "Test Player");
        assert!(json.get("life").is_some());
        assert!(json.get("strength").is_some());
        assert!(json.get("location").is_some());
    }
}