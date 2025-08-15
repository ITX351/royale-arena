use actix_web::{HttpResponse, Result, web};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ResetGameResponse {
    pub success: bool,
    pub message: String,
}

// 重置游戏
pub async fn reset_game(
    path: web::Path<String>, // game_id
    _data: web::Data<std::sync::Arc<tokio::sync::Mutex<crate::AppState>>>,
) -> Result<HttpResponse> {
    let _game_id = path.into_inner();

    // 在实际实现中，我们需要：
    // 1. 验证导演权限
    // 2. 重置游戏状态到初始状态

    // 目前返回示例响应
    let response = ResetGameResponse {
        success: true,
        message: "Game reset successfully".to_string(),
    };

    Ok(HttpResponse::Ok().json(response))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{create_test_app, create_test_app_state};
    use actix_web::{test, web};
    use serde_json::Value;

    #[actix_web::test]
    async fn test_reset_game() {
        // Create test app state and app
        let app_state = create_test_app_state();
        let app = test::init_service(
            create_test_app(app_state.clone())
                .route("/game/{game_id}/reset", web::post().to(reset_game)),
        )
        .await;

        // Make request
        let req = test::TestRequest::post()
            .uri("/game/test_game/reset")
            .to_request();
        let resp = test::call_service(&app, req).await;

        // Check response
        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
        let json: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["success"], true);
        assert_eq!(json["message"], "Game reset successfully");
    }
}
