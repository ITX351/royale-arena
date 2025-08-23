use actix_web::{HttpResponse, Result, web};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ExportGameResponse {
    pub success: bool,
    pub message: String,
    pub download_url: Option<String>,
}

// 导出游戏数据
pub async fn export_game_data(
    path: web::Path<String>, // game_id
    _data: web::Data<std::sync::Arc<tokio::sync::Mutex<crate::AppState>>>,
) -> Result<HttpResponse> {
    let game_id = path.into_inner();

    // 在实际实现中，我们需要：
    // 1. 验证导演权限
    // 2. 导出游戏数据并生成下载链接

    // 目前返回示例响应
    let response = ExportGameResponse {
        success: true,
        message: "Game data exported successfully".to_string(),
        download_url: Some(format!("/downloads/game-{}.zip", game_id)),
    };

    Ok(HttpResponse::Ok().json(response))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_common::test_utils::{create_test_app, create_test_app_state};
    use actix_web::{test, web};
    use serde_json::Value;

    #[actix_web::test]
    async fn test_export_game_data() {
        // Create test app state and app
        let app_state = create_test_app_state();
        let app = test::init_service(
            create_test_app(app_state.clone())
                .route("/game/{game_id}/export", web::get().to(export_game_data)),
        )
        .await;

        // Make request
        let req = test::TestRequest::get()
            .uri("/game/test_game/export")
            .to_request();
        let resp = test::call_service(&app, req).await;

        // Check response
        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
        let json: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["success"], true);
        assert_eq!(json["message"], "Game data exported successfully");
        assert!(json.get("download_url").is_some());
    }
}
