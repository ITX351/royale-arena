use axum::{
    extract::{Path, Query, State, Request},
    response::Json,
};
use serde_json::json;

use crate::routes::AppState;
use crate::admin::models::JwtClaims;
use super::errors::GameError;
use super::models::*;

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
    
    let games = state.game_service.get_games(&query, has_admin_privileges).await?;
    
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
    
    let game = state.game_service.get_game_with_rules(&game_id, has_admin_privileges).await?;
    
    Ok(Json(json!({
        "success": true,
        "data": game
    })))
}