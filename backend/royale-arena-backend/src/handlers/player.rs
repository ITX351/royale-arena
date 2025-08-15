use crate::models::player::Player;
use actix_web::{HttpResponse, Result, web};

/// 验证玩家凭证
fn verify_player_credentials(player: &Player, password: &str) -> bool {
    match player.verify_password(password) {
        Ok(valid) => valid,
        Err(_) => false,
    }
}

pub async fn get_player_info(
    path: web::Path<(String, String)>, // (game_id, player_id)
    data: web::Data<std::sync::Arc<tokio::sync::Mutex<crate::AppState>>>,
) -> Result<HttpResponse> {
    let (game_id, player_id) = path.into_inner();

    // 首先尝试从内存状态中获取玩家信息
    {
        let state = data.lock().await;
        // 在实际实现中，我们需要从游戏数据中查找玩家信息
        // 这里检查是否有对应游戏和玩家的数据
        if let Some(_game) = state.games.get(&game_id) {
            // TODO: 实现从游戏数据中获取真实玩家信息的逻辑
            // 这里应该查找特定玩家的数据
        }
    }

    // 在实际实现中，我们需要从游戏数据中查找玩家信息
    // 暂时返回一个示例玩家信息
    let player = Player::new(
        player_id.clone(),
        "Test Player".to_string(),
        "password123".to_string(),
        0, // 默认无队伍
    )
    .unwrap_or_else(|_| Player {
        id: player_id.clone(),
        name: "Test Player".to_string(),
        password_hash: "default_hash".to_string(),
        team_id: 0,
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
    });

    Ok(HttpResponse::Ok().json(player))
}

/// 玩家登录端点
pub async fn player_login(
    path: web::Path<(String, String)>, // (game_id, player_id)
    credentials: web::Json<std::collections::HashMap<String, String>>,
    data: web::Data<std::sync::Arc<tokio::sync::Mutex<crate::AppState>>>,
) -> Result<HttpResponse> {
    let (game_id, player_id) = path.into_inner();
    let password = credentials
        .get("password")
        .unwrap_or(&"".to_string())
        .clone();

    // 首先尝试从内存状态中获取玩家信息
    {
        let state = data.lock().await;
        // 在实际实现中，我们需要从游戏数据中查找玩家信息
        // 这里检查是否有对应游戏和玩家的数据
        if let Some(_game) = state.games.get(&game_id) {
            // TODO: 实现从游戏数据中获取真实玩家信息的逻辑
            // 这里应该查找特定玩家的数据
        }
    }

    // 在实际实现中，我们需要从游戏数据中查找玩家信息并验证凭证
    // 暂时创建一个示例玩家用于演示
    let player = Player::new(
        player_id.clone(),
        "Test Player".to_string(),
        "password123".to_string(),
        0, // 默认无队伍
    )
    .unwrap_or_else(|_| Player {
        id: player_id.clone(),
        name: "Test Player".to_string(),
        password_hash: "default_hash".to_string(),
        team_id: 0,
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
    });

    // 验证玩家凭证
    if verify_player_credentials(&player, &password) {
        // 在实际实现中，这里应该生成一个JWT令牌或其他身份验证令牌
        let token = uuid::Uuid::new_v4().to_string();

        Ok(HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "token": token,
            "expires_in": 3600
        })))
    } else {
        Ok(HttpResponse::Unauthorized().json(serde_json::json!({
            "success": false,
            "error": "无效的凭证"
        })))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{create_test_app, create_test_app_state};
    use actix_web::{test, web};
    use serde_json::Value;

    #[actix_web::test]
    async fn test_get_player_info() {
        // Create test app state and app
        let app_state = create_test_app_state();
        let app = test::init_service(create_test_app(app_state.clone()).route(
            "/game/{game_id}/player/{player_id}",
            web::get().to(get_player_info),
        ))
        .await;

        // Make request for player info
        let req = test::TestRequest::get()
            .uri("/game/test_game/player/test_player")
            .to_request();
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
