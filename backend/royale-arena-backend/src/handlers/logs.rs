use actix_web::{HttpResponse, Result, web};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct LogEntry {
    pub id: String,
    pub level: String,
    pub message: String,
    pub player_id: Option<String>,
    pub timestamp: String,
}

#[derive(Serialize, Deserialize)]
pub struct GameLogsResponse {
    pub logs: Vec<LogEntry>,
}

// 获取游戏日志
pub async fn get_game_logs(
    path: web::Path<String>, // game_id
    _data: web::Data<std::sync::Arc<tokio::sync::Mutex<crate::AppState>>>,
) -> Result<HttpResponse> {
    let game_id = path.into_inner();

    // 在实际实现中，我们需要：
    // 1. 验证导演权限
    // 2. 从数据库获取该游戏的日志

    // 目前返回示例数据
    let logs = vec![
        LogEntry {
            id: format!("log-{}-1", game_id),
            level: "info".to_string(),
            message: "游戏开始".to_string(),
            player_id: None,
            timestamp: "2023-01-01T00:00:00Z".to_string(),
        },
        LogEntry {
            id: format!("log-{}-2", game_id),
            level: "info".to_string(),
            message: "玩家加入游戏".to_string(),
            player_id: Some(format!("player-{}-1", game_id)),
            timestamp: "2023-01-01T00:01:00Z".to_string(),
        },
    ];

    let response = GameLogsResponse { logs };

    Ok(HttpResponse::Ok().json(response))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_common::test_utils::{create_test_app, create_test_app_state};
    use actix_web::{test, web};
    use serde_json::Value;

    #[actix_web::test]
    async fn test_get_game_logs() {
        // Create test app state and app
        let app_state = create_test_app_state();
        let app = test::init_service(
            create_test_app(app_state.clone())
                .route("/game/{game_id}/logs", web::get().to(get_game_logs)),
        )
        .await;

        // Make request
        let req = test::TestRequest::get()
            .uri("/game/test_game/logs")
            .to_request();
        let resp = test::call_service(&app, req).await;

        // Check response
        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
        let json: Value = serde_json::from_slice(&body).unwrap();

        assert!(json.get("logs").is_some());
        assert_eq!(json["logs"].as_array().unwrap().len(), 2);
    }
}
