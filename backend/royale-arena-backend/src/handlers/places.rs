use crate::models::place::{Place, PlaceStatus};
use actix_web::{HttpResponse, Result, web};

pub async fn get_places_status(
    path: web::Path<String>, // game_id
    data: web::Data<std::sync::Arc<tokio::sync::Mutex<crate::AppState>>>,
) -> Result<HttpResponse> {
    let game_id = path.into_inner();
    let state = data.lock().await;

    // 尝试从AppState中获取地点信息
    if let Some(places) = state.places.get(&game_id) {
        // 将Place转换为PlaceStatus
        let places_status: Vec<PlaceStatus> = places
            .iter()
            .map(|place| PlaceStatus::new(place.clone()))
            .collect();

        return Ok(HttpResponse::Ok().json(places_status));
    }

    // 如果没有找到特定游戏的地点信息，返回默认地点列表
    let default_places = crate::models::rules::GameRules::default()
        .places
        .iter()
        .enumerate()
        .map(|(index, name)| {
            let place = Place::new(
                format!("place_{}", index),
                name.clone(),
                format!("{}地点", name),
            );
            PlaceStatus::new(place)
        })
        .collect::<Vec<PlaceStatus>>();

    Ok(HttpResponse::Ok().json(default_places))
}

/// 更新地点状态
pub async fn update_place_status(
    path: web::Path<(String, String)>, // (game_id, place_id)
    update_data: web::Json<std::collections::HashMap<String, serde_json::Value>>,
    data: web::Data<std::sync::Arc<tokio::sync::Mutex<crate::AppState>>>,
) -> Result<HttpResponse> {
    let (game_id, place_id) = path.into_inner();
    let update_fields = update_data.into_inner();

    // 在实际实现中，这里应该更新地点状态
    // 目前我们只是模拟更新操作

    // 首先检查游戏是否存在
    {
        let state = data.lock().await;
        if !state.games.contains_key(&game_id) {
            return Ok(HttpResponse::NotFound().json(serde_json::json!({
                "error": "游戏未找到"
            })));
        }
    }

    // 模拟更新地点状态
    let mut response_data = std::collections::HashMap::new();
    response_data.insert("game_id", serde_json::Value::String(game_id));
    response_data.insert("place_id", serde_json::Value::String(place_id));
    response_data.insert(
        "updated_fields",
        serde_json::Value::Object(serde_json::Map::new()),
    );

    for (key, value) in update_fields {
        // 在实际实现中，这里应该验证和应用更新
        if let Some(serde_json::Value::Object(obj)) = response_data.get_mut("updated_fields") {
            obj.insert(key, value);
        }
    }

    Ok(HttpResponse::Ok().json(response_data))
}

/// 添加玩家到地点
pub async fn add_player_to_place(
    path: web::Path<(String, String, String)>, // (game_id, place_id, player_id)
    data: web::Data<std::sync::Arc<tokio::sync::Mutex<crate::AppState>>>,
) -> Result<HttpResponse> {
    let (game_id, place_id, player_id) = path.into_inner();

    // 在实际实现中，这里应该将玩家添加到指定地点
    // 目前我们只是模拟操作

    // 首先检查游戏是否存在
    {
        let state = data.lock().await;
        if !state.games.contains_key(&game_id) {
            return Ok(HttpResponse::NotFound().json(serde_json::json!({
                "error": "游戏未找到"
            })));
        }
    }

    // 模拟添加玩家到地点
    let response = serde_json::json!({
        "success": true,
        "game_id": game_id,
        "place_id": place_id,
        "player_id": player_id,
        "message": format!("玩家 {} 已添加到地点 {}", player_id, place_id)
    });

    Ok(HttpResponse::Ok().json(response))
}

/// 从地点移除玩家
pub async fn remove_player_from_place(
    path: web::Path<(String, String, String)>, // (game_id, place_id, player_id)
    data: web::Data<std::sync::Arc<tokio::sync::Mutex<crate::AppState>>>,
) -> Result<HttpResponse> {
    let (game_id, place_id, player_id) = path.into_inner();

    // 在实际实现中，这里应该从地点移除玩家
    // 目前我们只是模拟操作

    // 首先检查游戏是否存在
    {
        let state = data.lock().await;
        if !state.games.contains_key(&game_id) {
            return Ok(HttpResponse::NotFound().json(serde_json::json!({
                "error": "游戏未找到"
            })));
        }
    }

    // 模拟从地点移除玩家
    let response = serde_json::json!({
        "success": true,
        "game_id": game_id,
        "place_id": place_id,
        "player_id": player_id,
        "message": format!("玩家 {} 已从地点 {} 移除", player_id, place_id)
    });

    Ok(HttpResponse::Ok().json(response))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_common::test_utils::{create_test_app, create_test_app_state};
    use actix_web::{test, web};
    use serde_json::Value;

    #[actix_web::test]
    async fn test_get_places_status_default() {
        // Create test app state and app
        let app_state = create_test_app_state();
        let app = test::init_service(
            create_test_app(app_state.clone())
                .route("/game/{game_id}/places", web::get().to(get_places_status)),
        )
        .await;

        // Make request for places status
        let req = test::TestRequest::get()
            .uri("/game/test_game/places")
            .to_request();
        let resp = test::call_service(&app, req).await;

        // Check response
        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
        let json: Value = serde_json::from_slice(&body).unwrap();

        // Check that we got an array of places
        assert!(json.as_array().is_some());
        assert!(!json.as_array().unwrap().is_empty());
    }
}
