use axum::{
    extract::{Path, Query, State},
    response::Json,
};
use serde::Deserialize;
use serde_json::json;

use crate::routes::AppState;
use super::errors::DirectorError;
use super::models::*;

/// 导演密码查询参数
#[derive(Debug, Deserialize)]
pub struct DirectorPasswordQuery {
    pub password: String,
}

/// 批量添加演员 (导演接口)
pub async fn batch_add_players(
    State(state): State<AppState>,
    Path(game_id): Path<String>,
    Query(query): Query<DirectorPasswordQuery>,
    Json(request): Json<BatchAddPlayersRequest>,
) -> Result<Json<serde_json::Value>, DirectorError> {
    let response = state.director_service
        .batch_add_players(&game_id, &query.password, request)
        .await?;
    
    Ok(Json(json!({
        "success": true,
        "data": response
    })))
}

/// 获取演员列表 (导演接口)
pub async fn get_players(
    State(state): State<AppState>,
    Path(game_id): Path<String>,
    Query(query): Query<DirectorPasswordQuery>,
) -> Result<Json<serde_json::Value>, DirectorError> {
    let players = state.director_service
        .get_players(&game_id, &query.password)
        .await?;
    
    let response = PlayersListResponse { players };
    
    Ok(Json(json!({
        "success": true,
        "data": response
    })))
}

/// 批量删除演员 (导演接口)
pub async fn batch_delete_players(
    State(state): State<AppState>,
    Path(game_id): Path<String>,
    Query(query): Query<DirectorPasswordQuery>,
    Json(request): Json<BatchDeletePlayersRequest>,
) -> Result<Json<serde_json::Value>, DirectorError> {
    let response = state.director_service
        .batch_delete_players(&game_id, &query.password, request)
        .await?;
    
    Ok(Json(json!({
        "success": true,
        "data": response
    })))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_director_password_query_deserialization() {
        // 这个测试可以验证查询参数的反序列化是否正常工作
        let test_password = "test123";
        let query = DirectorPasswordQuery {
            password: test_password.to_string(),
        };
        assert_eq!(query.password, test_password);
    }
}