use crate::models::rules::GameRules;
use actix_web::{HttpResponse, Result, web};

pub async fn get_game_rules(
    path: web::Path<String>,
    data: web::Data<std::sync::Arc<tokio::sync::Mutex<crate::AppState>>>,
) -> Result<HttpResponse> {
    let game_id = path.into_inner();
    let state = data.lock().await;

    match state.game_rules.get(&game_id) {
        Some(rules) => Ok(HttpResponse::Ok().json(rules)),
        None => {
            // 如果没有找到特定游戏的规则，返回默认规则
            let default_rules = GameRules::default();
            Ok(HttpResponse::Ok().json(default_rules))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_common::test_utils::{create_test_app, create_test_app_state};
    use actix_web::{test, web};
    use serde_json::Value;

    #[actix_web::test]
    async fn test_get_game_rules_default() {
        // Create test app state and app
        let app_state = create_test_app_state();
        let app = test::init_service(
            create_test_app(app_state.clone())
                .route("/game/{game_id}/rules", web::get().to(get_game_rules)),
        )
        .await;

        // Make request for game rules
        let req = test::TestRequest::get()
            .uri("/game/test_game/rules")
            .to_request();
        let resp = test::call_service(&app, req).await;

        // Check response
        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
        let json: Value = serde_json::from_slice(&body).unwrap();

        // Check that we got a valid rules object
        assert!(json.get("max_life").is_some());
        assert!(json.get("max_strength").is_some());
        assert!(json.get("places").is_some());
    }
}
