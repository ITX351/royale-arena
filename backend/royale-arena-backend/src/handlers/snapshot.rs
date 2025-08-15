use crate::models::game::Game;
use crate::models::player::Player;
use actix_web::{HttpResponse, Result, web};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct GameSnapshot {
    pub game: Game,
    pub players: Vec<Player>,
    pub timestamp: String,
}

#[derive(Serialize, Deserialize)]
pub struct GameSnapshotResponse {
    pub snapshot: GameSnapshot,
}

// 获取游戏完整状态快照
pub async fn get_game_snapshot(
    path: web::Path<String>, // game_id
    _data: web::Data<std::sync::Arc<tokio::sync::Mutex<crate::AppState>>>,
) -> Result<HttpResponse> {
    let game_id = path.into_inner();

    // 在实际实现中，我们需要：
    // 1. 验证导演权限
    // 2. 获取游戏的完整状态快照

    // 目前返回示例数据
    let game = Game {
        id: game_id.clone(),
        name: "测试游戏".to_string(),
        description: "这是一个测试游戏".to_string(),
        status: "running".to_string(),
        rule_template_id: Some("default".to_string()),
        phase: "day".to_string(),
        player_count: 50,
        max_players: 100,
        start_time: Some("2023-01-01T00:00:00Z".to_string()),
        end_time: None,
        action_start_time: Some("2023-01-01T01:00:00Z".to_string()),
        action_end_time: Some("2023-01-01T02:00:00Z".to_string()),
        safe_zones: vec!["安全区1".to_string(), "安全区2".to_string()],
        weather: 0.5,
        announcements: vec!["欢迎来到测试游戏!".to_string()],
    };

    let players = vec![
        Player::new(
            format!("player-{}-1", game_id),
            "测试玩家1".to_string(),
            "password123".to_string(),
            0,
        )
        .unwrap_or_else(|_| {
            // 如果创建失败，使用默认值
            Player {
                id: format!("player-{}-1", game_id),
                name: "测试玩家1".to_string(),
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
            }
        }),
    ];

    let snapshot = GameSnapshot {
        game,
        players,
        timestamp: "2023-01-01T01:30:00Z".to_string(),
    };

    let response = GameSnapshotResponse { snapshot };

    Ok(HttpResponse::Ok().json(response))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{create_test_app, create_test_app_state};
    use actix_web::{test, web};
    use serde_json::Value;

    #[actix_web::test]
    async fn test_get_game_snapshot() {
        // Create test app state and app
        let app_state = create_test_app_state();
        let app = test::init_service(
            create_test_app(app_state.clone())
                .route("/game/{game_id}/snapshot", web::get().to(get_game_snapshot)),
        )
        .await;

        // Make request
        let req = test::TestRequest::get()
            .uri("/game/test_game/snapshot")
            .to_request();
        let resp = test::call_service(&app, req).await;

        // Check response
        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
        let json: Value = serde_json::from_slice(&body).unwrap();

        assert!(json.get("snapshot").is_some());
        assert!(json["snapshot"].get("game").is_some());
        assert!(json["snapshot"].get("players").is_some());
    }
}
