use actix_web::{HttpResponse, Result, web};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct GameStats {
    pub player_count: u32,
    pub alive_players: u32,
    pub total_actions: u32,
    pub start_time: Option<String>,
    pub duration: u32,
    pub votes_cast: u32,
    pub eliminations: u32,
}

#[derive(Serialize, Deserialize)]
pub struct GameStatsResponse {
    pub stats: GameStats,
}

// 获取游戏统计
pub async fn get_game_stats(
    path: web::Path<String>, // game_id
    _data: web::Data<std::sync::Arc<tokio::sync::Mutex<crate::AppState>>>,
) -> Result<HttpResponse> {
    let _game_id = path.into_inner();

    // 在实际实现中，我们需要：
    // 1. 验证导演权限
    // 2. 从数据库或内存中获取游戏统计数据

    // 目前返回示例数据
    let stats = GameStats {
        player_count: 50,
        alive_players: 30,
        total_actions: 1200,
        start_time: Some("2023-01-01T00:00:00Z".to_string()),
        duration: 3600,
        votes_cast: 45,
        eliminations: 20,
    };

    let response = GameStatsResponse { stats };

    Ok(HttpResponse::Ok().json(response))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_common::test_utils::{create_test_app, create_test_app_state};
    use actix_web::{test, web};
    use serde_json::Value;

    #[actix_web::test]
    async fn test_get_game_stats() {
        // Create test app state and app
        let app_state = create_test_app_state();
        let app = test::init_service(
            create_test_app(app_state.clone())
                .route("/game/{game_id}/stats", web::get().to(get_game_stats)),
        )
        .await;

        // Make request
        let req = test::TestRequest::get()
            .uri("/game/test_game/stats")
            .to_request();
        let resp = test::call_service(&app, req).await;

        // Check response
        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
        let json: Value = serde_json::from_slice(&body).unwrap();

        assert!(json.get("stats").is_some());
        assert_eq!(json["stats"]["player_count"], 50);
        assert_eq!(json["stats"]["alive_players"], 30);
    }
}
