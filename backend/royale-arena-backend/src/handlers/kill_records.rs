use actix_web::{HttpResponse, Result, web};
use serde::{Deserialize, Serialize};
use crate::services::kill_record_service::get_kill_records_from_db;
use crate::services::actor_service::verify_director_password;

#[derive(Serialize, Deserialize)]
pub struct KillRecordResponse {
    pub killer: Option<String>,  // 击杀者名称（可为空）
    pub victim: String,          // 被击杀者名称
    pub kill_time: String,       // 击杀时间（ISO8601格式）
    pub cause: String,           // 击杀原因
    pub weapon: Option<String>,  // 使用的武器
    pub location: Option<String>, // 击杀地点
}

#[derive(Serialize, Deserialize)]
pub struct GameKillsResponse {
    pub kills: Vec<KillRecordResponse>,
    pub total_kills: u32,
    pub most_kills: Option<String>, // 击杀数最多的玩家（平票时为null）
}

// 获取击杀记录
pub async fn get_game_kills(
    path: web::Path<(String, String)>, // (game_id, password)
    _data: web::Data<std::sync::Arc<tokio::sync::Mutex<crate::AppState>>>,
) -> Result<HttpResponse> {
    let (game_id, password) = path.into_inner();
    
    // 验证导演密码
    if !verify_director_password(&game_id, &password).await? {
        return Ok(HttpResponse::Unauthorized().json("Invalid director password"));
    }

    // 从数据库获取击杀记录
    let kill_records = get_kill_records_from_db(&game_id).await?;

    // 转换为响应格式
    let kills: Vec<KillRecordResponse> = kill_records.iter().map(|kill_record| {
        KillRecordResponse {
            killer: kill_record.killer_id.clone(),
            victim: kill_record.victim_id.clone(),
            kill_time: kill_record.kill_time.to_rfc3339(),
            cause: kill_record.cause.clone(),
            weapon: kill_record.weapon.clone(),
            location: kill_record.location.clone(),
        }
    }).collect();

    // 计算统计数据
    let total_kills = kills.len() as u32;
    
    // 计算击杀数最多的玩家
    let mut kill_counts = std::collections::HashMap::new();
    for kill in &kills {
        if let Some(killer) = &kill.killer {
            *kill_counts.entry(killer.clone()).or_insert(0) += 1;
        }
    }
    
    let most_kills = if kill_counts.is_empty() {
        None
    } else {
        let max_kills = *kill_counts.values().max().unwrap();
        let top_killers: Vec<_> = kill_counts.iter()
            .filter(|&(_, &count)| count == max_kills)
            .map(|(name, _)| name)
            .collect();
        
        if top_killers.len() == 1 {
            Some(top_killers[0].clone())
        } else {
            None // 平票情况
        }
    };

    let response = GameKillsResponse {
        kills,
        total_kills,
        most_kills,
    };

    Ok(HttpResponse::Ok().json(response))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_common::test_utils::{create_test_app, create_test_app_state};
    use crate::test_common::test_data::TestDataManager;
    use crate::test_common::test_init::init_test_env;
    use actix_web::{test, web};
    use serde_json::Value;

    #[actix_web::test]
    async fn test_get_game_kills() {
        // Initialize test environment
        init_test_env();
        
        // Create test data manager
        let mut test_data_manager = TestDataManager::new();
        
        // Create a test game with a known password
        let game_id = test_data_manager
            .create_test_game("Test Game", "A test game", "director123", 50)
            .expect("Failed to create test game");
        
        // Create test app state and app
        let app_state = create_test_app_state();
        let app = test::init_service(
            create_test_app(app_state.clone())
                .route("/director/{game_id}/kills/{password}", web::get().to(get_game_kills)),
        )
        .await;

        // Make request with the correct game ID and password
        let req = test::TestRequest::get()
            .uri(&format!("/director/{}/kills/director123", game_id))
            .to_request();
        let resp = test::call_service(&app, req).await;

        // Check response - should be successful since we created a game with the correct password
        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
        let json: Value = serde_json::from_slice(&body).unwrap();

        assert!(json.get("kills").is_some());
        assert!(json.get("total_kills").is_some());
        assert!(json.get("most_kills").is_some());
        
        // Clean up test data
        test_data_manager.cleanup().expect("Failed to cleanup test data");
    }
}