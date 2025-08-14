use actix_web::{web, HttpResponse, Result};
use crate::models::place::{Place, PlaceStatus};

pub async fn get_places_status(
    path: web::Path<String>, // game_id
    data: web::Data<std::sync::Arc<tokio::sync::Mutex<crate::AppState>>>
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
    let default_places = crate::models::rules::GameRules::default().places
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

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, web};
    use serde_json::Value;
    use crate::test_utils::{create_test_app, create_test_app_state};

    #[actix_web::test]
    async fn test_get_places_status_default() {
        // Create test app state and app
        let app_state = create_test_app_state();
        let app = test::init_service(
            create_test_app(app_state.clone())
                .route("/game/{game_id}/places", web::get().to(get_places_status))
        ).await;

        // Make request for places status
        let req = test::TestRequest::get().uri("/game/test_game/places").to_request();
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