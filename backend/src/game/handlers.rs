use axum::{
    extract::{Path, Query, Request, State},
    response::Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;

use super::errors::GameError;
use super::models::*;
use crate::admin::models::JwtClaims;
use crate::routes::AppState;

/// 导演密码查询参数
#[derive(Debug, Deserialize)]
pub struct DirectorPasswordQuery {
    pub password: String,
}

/// 删除日志的时间戳参数
#[derive(Debug, Deserialize)]
pub struct DeleteLogsQuery {
    pub after_timestamp: Option<String>,
}

/// 创建游戏 (管理员接口)
pub async fn create_game(
    State(state): State<AppState>,
    Json(request): Json<CreateGameRequest>,
) -> Result<Json<serde_json::Value>, GameError> {
    let game = state.game_service.create_game(request).await?;

    Ok(Json(json!({
        "success": true,
        "data": game
    })))
}

/// 更新游戏 (管理员接口)
pub async fn update_game(
    State(state): State<AppState>,
    Path(game_id): Path<String>,
    Json(request): Json<UpdateGameRequest>,
) -> Result<Json<serde_json::Value>, GameError> {
    let game = state.game_service.update_game(&game_id, request).await?;

    Ok(Json(json!({
        "success": true,
        "data": game
    })))
}

/// 删除游戏 (管理员接口)
pub async fn delete_game(
    State(state): State<AppState>,
    Path(game_id): Path<String>,
) -> Result<Json<serde_json::Value>, GameError> {
    state.game_service.delete_game(&game_id).await?;

    Ok(Json(json!({
        "success": true,
        "message": "Game deleted successfully"
    })))
}

/// 获取游戏列表 (公开接口)
pub async fn get_games(
    State(state): State<AppState>,
    Query(query): Query<GameListQuery>,
    req: Request,
) -> Result<Json<serde_json::Value>, GameError> {
    // 检查是否有管理员权限
    let has_admin_privileges = req.extensions().get::<JwtClaims>().is_some();

    let games = state
        .game_service
        .get_games(&query, has_admin_privileges)
        .await?;

    Ok(Json(json!({
        "success": true,
        "data": games
    })))
}

/// 获取指定游戏信息（包括规则） (公开接口)
pub async fn get_game_with_rules(
    State(state): State<AppState>,
    Path(game_id): Path<String>,
    req: Request,
) -> Result<Json<serde_json::Value>, GameError> {
    // 检查是否有管理员权限
    let has_admin_privileges = req.extensions().get::<JwtClaims>().is_some();

    let game = state
        .game_service
        .get_game_by_id_with_player_counts(&game_id, has_admin_privileges)
        .await?;

    Ok(Json(json!({
        "success": true,
        "data": game
    })))
}

/// 获取玩家消息记录 (玩家接口)
pub async fn get_player_messages(
    State(state): State<AppState>,
    Path((game_id, player_id)): Path<(String, String)>,
    Json(request): Json<GetPlayerMessagesRequest>,
) -> Result<Json<serde_json::Value>, GameError> {
    // 验证请求参数
    request.validate().map_err(GameError::ValidationError)?;

    // 获取玩家消息记录
    let messages = state
        .game_log_service
        .get_player_messages(&game_id, &player_id, &request.password)
        .await?;

    Ok(Json(json!({
        "success": true,
        "data": messages
    })))
}

/// 获取导演消息记录 (导演接口)
pub async fn get_director_messages(
    State(state): State<AppState>,
    Path(game_id): Path<String>,
    Query(query): Query<DirectorPasswordQuery>,
) -> Result<Json<serde_json::Value>, GameError> {
    // 获取导演消息记录
    let messages = state
        .game_log_service
        .get_director_messages(&game_id, &query.password)
        .await?;

    Ok(Json(json!({
        "success": true,
        "data": messages
    })))
}

/// 删除游戏日志记录 (管理员接口)
pub async fn delete_game_logs(
    State(state): State<AppState>,
    Path(game_id): Path<String>,
    Query(query): Query<DeleteLogsQuery>,
) -> Result<Json<serde_json::Value>, GameError> {
    // 解析时间戳参数
    let timestamp = if let Some(ts_str) = query.after_timestamp {
        Some(
            chrono::DateTime::parse_from_rfc3339(&ts_str)
                .map_err(|_| GameError::ValidationError("Invalid timestamp format".to_string()))?
                .with_timezone(&chrono::Utc),
        )
    } else {
        None
        };

    // 删除日志记录
    let deleted_count = state
        .game_log_service
        .delete_logs_after_timestamp(&game_id, timestamp)
        .await?;

    Ok(Json(json!({
        "success": true,
        "message": format!("Deleted {} log records", deleted_count)
    })))
}

/// 游戏身份验证处理函数
pub async fn authenticate_game(
    State(state): State<AppState>,
    Path(game_id): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<String>, GameError> {
    let password = params
        .get("password")
        .ok_or_else(|| GameError::ValidationError("Password is required".to_string()))?;

    let result = state
        .director_service
        .authenticate_game(&game_id, password)
        .await
        .map_err(|e| GameError::OtherError(format!("Authentication failed: {}", e)))?;

    Ok(Json(result))
}