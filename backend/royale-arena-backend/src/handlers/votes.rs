use actix_web::{HttpResponse, Result, web};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct VoteRecord {
    pub id: String,
    pub voter_id: String,
    pub target_id: String,
    pub timestamp: String,
}

#[derive(Serialize, Deserialize)]
pub struct VoteResult {
    pub player_id: String,
    pub votes_received: u32,
}

#[derive(Serialize, Deserialize)]
pub struct GameVotesResponse {
    pub votes: Vec<VoteRecord>,
    pub results: Vec<VoteResult>,
}

// 获取投票结果
pub async fn get_game_votes(
    path: web::Path<String>, // game_id
    _data: web::Data<std::sync::Arc<tokio::sync::Mutex<crate::AppState>>>,
) -> Result<HttpResponse> {
    let game_id = path.into_inner();

    // 在实际实现中，我们需要：
    // 1. 验证导演权限
    // 2. 从数据库获取投票记录和统计结果

    // 目前返回示例数据
    let votes = vec![
        VoteRecord {
            id: format!("vote-{}-1", game_id),
            voter_id: format!("player-{}-1", game_id),
            target_id: format!("player-{}-2", game_id),
            timestamp: "2023-01-01T01:00:00Z".to_string(),
        },
        VoteRecord {
            id: format!("vote-{}-2", game_id),
            voter_id: format!("player-{}-3", game_id),
            target_id: format!("player-{}-2", game_id),
            timestamp: "2023-01-01T01:05:00Z".to_string(),
        },
    ];

    let results = vec![VoteResult {
        player_id: format!("player-{}-2", game_id),
        votes_received: 2,
    }];

    let response = GameVotesResponse { votes, results };

    Ok(HttpResponse::Ok().json(response))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_common::test_utils::{create_test_app, create_test_app_state};
    use actix_web::{test, web};
    use serde_json::Value;

    #[actix_web::test]
    async fn test_get_game_votes() {
        // Create test app state and app
        let app_state = create_test_app_state();
        let app = test::init_service(
            create_test_app(app_state.clone())
                .route("/game/{game_id}/votes", web::get().to(get_game_votes)),
        )
        .await;

        // Make request
        let req = test::TestRequest::get()
            .uri("/game/test_game/votes")
            .to_request();
        let resp = test::call_service(&app, req).await;

        // Check response
        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
        let json: Value = serde_json::from_slice(&body).unwrap();

        assert!(json.get("votes").is_some());
        assert!(json.get("results").is_some());
        assert_eq!(json["votes"].as_array().unwrap().len(), 2);
        assert_eq!(json["results"].as_array().unwrap().len(), 1);
    }
}
