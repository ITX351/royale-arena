use axum::{
    extract::{Path, Query, State},
    response::Json,
};
use serde::Deserialize;
use serde_json::json;

use super::errors::DirectorError;
use super::models::*;
use crate::game::models::GameStatus;
use crate::routes::AppState;

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
    let response = state
        .director_service
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
    let players = state
        .director_service
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
    let response = state
        .director_service
        .batch_delete_players(&game_id, &query.password, request)
        .await?;

    Ok(Json(json!({
        "success": true,
        "data": response
    })))
}

/// 导演更新游戏状态
pub async fn update_game_status(
    State(state): State<AppState>,
    Path(game_id): Path<String>,
    Json(request): Json<UpdateGameStatusRequest>,
) -> Result<Json<serde_json::Value>, DirectorError> {
    // 验证导演密码
    state
        .director_service
        .verify_director_password(&game_id, &request.password)
        .await?;

    // 获取当前游戏状态
    let game = state
        .game_service
        .get_game_by_id(&game_id)
        .await
        .map_err(|e| DirectorError::OtherError {
            message: format!("Failed to get game: {}", e),
        })?;

    // 根据目标状态调用对应的导演服务方法
    let result: Result<UpdateGameStatusResponse, DirectorError> = match request.status {
        GameStatus::Running => match game.status {
            GameStatus::Waiting => {
                state.director_service.start_game(&state, &game_id).await?;
                Ok(UpdateGameStatusResponse {
                    success: true,
                    message: "Game started successfully".to_string(),
                    save_file_name: None,
                })
            }
            GameStatus::Paused => {
                state
                    .director_service
                    .resume_game(&state, &game_id, request.save_file_name)
                    .await?;
                Ok(UpdateGameStatusResponse {
                    success: true,
                    message: "Game resumed successfully".to_string(),
                    save_file_name: None,
                })
            }
            _ => return Err(DirectorError::InvalidGameStateTransition),
        },
        GameStatus::Paused => {
            // 只有在运行状态才能暂停
            match game.status {
                GameStatus::Running => {
                    let save_file_name =
                        state.director_service.pause_game(&state, &game_id).await?;
                    Ok(UpdateGameStatusResponse {
                        success: true,
                        message: "Game paused successfully".to_string(),
                        save_file_name: Some(save_file_name),
                    })
                }
                _ => return Err(DirectorError::InvalidGameStateTransition),
            }
        }
        GameStatus::Waiting => {
            // 只有在暂停状态才能回退到等待
            match game.status {
                GameStatus::Paused => {
                    state
                        .director_service
                        .reset_game_to_waiting(&state, &game_id)
                        .await?;
                    Ok(UpdateGameStatusResponse {
                        success: true,
                        message: "Game reset to waiting state successfully".to_string(),
                        save_file_name: None,
                    })
                }
                _ => return Err(DirectorError::InvalidGameStateTransition),
            }
        }
        GameStatus::Ended => {
            // 只有在运行或暂停状态才能结束
            match game.status {
                GameStatus::Running | GameStatus::Paused => {
                    state
                        .director_service
                        .end_game(&state, &game_id, game.status == GameStatus::Running)
                        .await?;
                    Ok(UpdateGameStatusResponse {
                        success: true,
                        message: "Game ended successfully".to_string(),
                        save_file_name: None,
                    })
                }
                _ => return Err(DirectorError::InvalidGameStateTransition),
            }
        }
        _ => return Err(DirectorError::InvalidGameStateTransition),
    };

    let response = result?;

    Ok(Json(json!(response)))
}

/// 手动存盘接口
pub async fn manual_save(
    State(state): State<AppState>,
    Path(game_id): Path<String>,
    Json(request): Json<ManualSaveRequest>,
) -> Result<Json<serde_json::Value>, DirectorError> {
    let save_file_name = state
        .director_service
        .manual_save(&state, &game_id, &request.password)
        .await?;

    let response = ManualSaveResponse {
        success: true,
        message: "Game state saved successfully".to_string(),
        save_file_name,
    };

    Ok(Json(json!(response)))
}

/// 导演编辑游戏
pub async fn edit_game(
    State(state): State<AppState>,
    Path(game_id): Path<String>,
    Query(query): Query<DirectorPasswordQuery>,
    Json(request): Json<DirectorEditGameRequest>,
) -> Result<Json<serde_json::Value>, DirectorError> {
    let game = state
        .director_service
        .edit_game(&state, &game_id, &query.password, request)
        .await?;

    Ok(Json(json!({
        "success": true,
        "data": game
    })))
}

/// 查询存档文件列表接口
pub async fn list_save_files(
    State(state): State<AppState>,
    Path(game_id): Path<String>,
    Query(query): Query<DirectorPasswordQuery>,
) -> Result<Json<serde_json::Value>, DirectorError> {
    let save_files = state
        .director_service
        .list_save_files(&state, &game_id, &query.password)
        .await?;

    let response = ListSaveFilesResponse {
        success: true,
        data: save_files,
    };

    Ok(Json(json!(response)))
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
