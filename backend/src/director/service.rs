use crate::director::{DirectorError, models::*};
use crate::game::models::{GameWithPlayerCounts, SaveFileInfo};
use crate::routes::AppState;
use sqlx::{MySqlPool, Row};
use uuid::Uuid;

/// 导演服务层
#[derive(Clone)]
pub struct DirectorService {
    pub pool: MySqlPool,
}

impl DirectorService {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    /// 获取导演明文密码
    async fn get_director_password(&self, game_id: &str) -> Result<String, DirectorError> {
        let row = sqlx::query("SELECT director_password FROM games WHERE id = ?")
            .bind(game_id)
            .fetch_optional(&self.pool)
            .await?;

        match row {
            Some(row) => {
                let stored_password: String = row.get("director_password");
                Ok(stored_password)
            }
            None => Err(DirectorError::GameNotFound),
        }
    }

    /// 验证导演密码
    pub async fn verify_director_password(
        &self,
        game_id: &str,
        password: &str,
    ) -> Result<(), DirectorError> {
        let director_password = self.get_director_password(game_id).await?;
        if director_password == password {
            Ok(())
        } else {
            Err(DirectorError::InvalidDirectorPassword)
        }
    }

    /// 检查游戏状态是否允许删除演员
    async fn check_game_status_for_deletion(&self, game_id: &str) -> Result<(), DirectorError> {
        let row = sqlx::query("SELECT status FROM games WHERE id = ?")
            .bind(game_id)
            .fetch_optional(&self.pool)
            .await?;

        match row {
            Some(row) => {
                let status: String = row.get("status");
                if status == "waiting" {
                    Ok(())
                } else {
                    Err(DirectorError::GameAlreadyStarted)
                }
            }
            None => Err(DirectorError::GameNotFound),
        }
    }

    /// 检查演员名称是否在游戏中重复
    async fn check_player_name_exists(
        &self,
        game_id: &str,
        name: &str,
    ) -> Result<bool, DirectorError> {
        let row =
            sqlx::query("SELECT COUNT(*) as count FROM actors WHERE game_id = ? AND name = ?")
                .bind(game_id)
                .bind(name)
                .fetch_one(&self.pool)
                .await?;

        let count: i64 = row.get("count");
        Ok(count > 0)
    }

    /// 创建单个演员（内部方法）
    async fn create_player(
        &self,
        game_id: &str,
        player_name: &str,
        password: &str,
        team_id: i32,
    ) -> Result<PlayerInfo, DirectorError> {
        let player_id = Uuid::new_v4().to_string();

        sqlx::query(
            "INSERT INTO actors (id, game_id, name, password, team_id, created_at, updated_at) 
             VALUES (?, ?, ?, ?, ?, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
        )
        .bind(&player_id)
        .bind(game_id)
        .bind(player_name)
        .bind(password)
        .bind(team_id)
        .execute(&self.pool)
        .await?;

        Ok(PlayerInfo {
            id: player_id,
            name: player_name.to_string(),
            password: password.to_string(),
            game_id: game_id.to_string(),
            team_id,
        })
    }

    /// 批量添加演员
    pub async fn batch_add_players(
        &self,
        game_id: &str,
        password: &str,
        request: BatchAddPlayersRequest,
    ) -> Result<BatchOperationResponse<PlayerInfo>, DirectorError> {
        // 验证导演密码
        self.verify_director_password(game_id, password).await?;

        // 获取导演密码用于后续验证
        let director_password = self.get_director_password(game_id).await?;

        // 首先检查所有演员密码是否与导演密码相同
        for player_request in &request.players {
            if player_request.password == director_password {
                // 如果发现任何演员密码与导演密码相同，拒绝所有演员的加入
                let mut failed = Vec::new();
                for player_request in request.players {
                    failed.push(OperationFailure {
                        player_name: Some(player_request.player_name.clone()),
                        id: None,
                        reason: "演员密码不能与导演密码相同".to_string(),
                    });
                }
                return Ok(BatchOperationResponse {
                    success: Vec::new(),
                    failed,
                });
            }
        }

        let mut success = Vec::new();
        let mut failed = Vec::new();

        for player_request in request.players {
            // 验证单个玩家数据
            if let Err(validation_error) = player_request.validate() {
                failed.push(OperationFailure {
                    player_name: Some(player_request.player_name.clone()),
                    id: None,
                    reason: validation_error,
                });
                continue;
            }

            // 检查名称重复
            match self
                .check_player_name_exists(game_id, &player_request.player_name)
                .await
            {
                Ok(exists) => {
                    if exists {
                        failed.push(OperationFailure {
                            player_name: Some(player_request.player_name.clone()),
                            id: None,
                            reason: format!("演员名称已存在: {}", player_request.player_name),
                        });
                        continue;
                    }
                }
                Err(e) => {
                    failed.push(OperationFailure {
                        player_name: Some(player_request.player_name.clone()),
                        id: None,
                        reason: format!("检查名称重复时发生错误: {}", e),
                    });
                    continue;
                }
            }

            // 创建演员
            match self
                .create_player(
                    game_id,
                    &player_request.player_name,
                    &player_request.password,
                    player_request.get_team_id(),
                )
                .await
            {
                Ok(player_info) => {
                    success.push(player_info);
                }
                Err(e) => {
                    failed.push(OperationFailure {
                        player_name: Some(player_request.player_name.clone()),
                        id: None,
                        reason: format!("创建演员失败: {}", e),
                    });
                }
            }
        }

        Ok(BatchOperationResponse { success, failed })
    }

    /// 获取演员列表
    pub async fn get_players(
        &self,
        game_id: &str,
        password: &str,
    ) -> Result<Vec<PlayerInfo>, DirectorError> {
        // 验证导演密码
        self.verify_director_password(game_id, password).await?;

        let players = sqlx::query_as::<_, PlayerInfo>(
            "SELECT id, game_id, name, password, team_id 
             FROM actors 
             WHERE game_id = ? 
             ORDER BY created_at ASC",
        )
        .bind(game_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(players)
    }

    /// 获取演员信息（内部方法）
    async fn get_player_by_id(&self, player_id: &str) -> Result<PlayerInfo, DirectorError> {
        let player = sqlx::query_as::<_, PlayerInfo>(
            "SELECT id, game_id, name, password, team_id FROM actors WHERE id = ?",
        )
        .bind(player_id)
        .fetch_optional(&self.pool)
        .await?;

        match player {
            Some(player) => Ok(player),
            None => Err(DirectorError::PlayerNotFound {
                id: player_id.to_string(),
            }),
        }
    }

    /// 删除单个演员（内部方法）
    async fn delete_player(&self, player_id: &str) -> Result<PlayerInfo, DirectorError> {
        // 先获取演员信息
        let player = self.get_player_by_id(player_id).await?;

        // 删除演员
        let result = sqlx::query("DELETE FROM actors WHERE id = ?")
            .bind(player_id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(DirectorError::PlayerNotFound {
                id: player_id.to_string(),
            });
        }

        Ok(player)
    }

    /// 批量删除演员
    pub async fn batch_delete_players(
        &self,
        game_id: &str,
        password: &str,
        request: BatchDeletePlayersRequest,
    ) -> Result<BatchOperationResponse<DeleteSuccessInfo>, DirectorError> {
        // 验证导演密码
        self.verify_director_password(game_id, password).await?;

        // 检查游戏状态
        self.check_game_status_for_deletion(game_id).await?;

        let mut success = Vec::new();
        let mut failed = Vec::new();

        for player_id in request.player_ids {
            // 验证玩家存在性并检查是否属于当前游戏
            match self.get_player_by_id(&player_id).await {
                Ok(player) => {
                    // 检查演员是否属于当前游戏
                    if player.game_id != game_id {
                        failed.push(OperationFailure {
                            player_name: None,
                            id: Some(player_id),
                            reason: "演员不属于指定游戏".to_string(),
                        });
                        continue;
                    }

                    // 执行删除操作
                    match self.delete_player(&player_id).await {
                        Ok(deleted_player) => {
                            success.push(DeleteSuccessInfo {
                                id: deleted_player.id,
                                name: deleted_player.name,
                                message: "Player deleted successfully".to_string(),
                            });
                        }
                        Err(e) => {
                            failed.push(OperationFailure {
                                player_name: None,
                                id: Some(player_id),
                                reason: format!("删除演员失败: {}", e),
                            });
                        }
                    }
                }
                Err(_) => {
                    failed.push(OperationFailure {
                        player_name: None,
                        id: Some(player_id),
                        reason: "演员不存在".to_string(),
                    });
                }
            }
        }

        Ok(BatchOperationResponse { success, failed })
    }

    /// 开始游戏（等待中 → 进行中）
    pub async fn start_game(
        &self,
        app_state: &AppState,
        game_id: &str,
    ) -> Result<(), DirectorError> {
        // 更新数据库中游戏状态为 "running"
        let result = sqlx::query!(
            "UPDATE games SET status = 'running', updated_at = CURRENT_TIMESTAMP WHERE id = ?",
            game_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| DirectorError::DatabaseError(e))?;

        if result.rows_affected() == 0 {
            return Err(DirectorError::GameNotFound);
        }

        // 初始化游戏内存状态
        let game = app_state
            .game_service
            .get_game_by_id(game_id)
            .await
            .map_err(|e| DirectorError::OtherError {
                message: format!("Failed to get game: {}", e),
            })?;
        app_state
            .game_state_manager
            .create_game_state(game_id, game.rules_config)
            .await
            .map_err(|e| DirectorError::OtherError {
                message: format!("Failed to create game state: {}", e),
            })?;

        Ok(())
    }

    /// 暂停游戏（进行中 → 暂停）
    pub async fn pause_game(
        &self,
        app_state: &AppState,
        game_id: &str,
    ) -> Result<String, DirectorError> {
        // 更新数据库中游戏状态为 "paused"
        let result = sqlx::query!(
            "UPDATE games SET status = 'paused', updated_at = CURRENT_TIMESTAMP WHERE id = ?",
            game_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| DirectorError::DatabaseError(e))?;

        if result.rows_affected() == 0 {
            return Err(DirectorError::GameNotFound);
        }

        // 断开所有WebSocket连接
        app_state
            .global_connection_manager
            .disconnect_game_connections(game_id, "游戏已暂停，连接已被服务器关闭。")
            .await;

        // 将当前游戏状态序列化并保存到磁盘文件
        let save_file_name = app_state
            .game_state_manager
            .save_game_state_to_disk(game_id)
            .await
            .map_err(|e| DirectorError::OtherError {
                message: format!("Failed to save game state to disk: {}", e),
            })?;

        Ok(save_file_name)
    }

    /// 回退游戏到等待状态（暂停 → 等待）
    pub async fn reset_game_to_waiting(
        &self,
        app_state: &AppState,
        game_id: &str,
    ) -> Result<(), DirectorError> {
        let result = sqlx::query!(
            "UPDATE games SET status = 'waiting', updated_at = CURRENT_TIMESTAMP WHERE id = ? AND status = 'paused'",
            game_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| DirectorError::DatabaseError(e))?;

        if result.rows_affected() == 0 {
            return Err(DirectorError::InvalidGameStateTransition);
        }

        // 移除内存中的游戏状态，使游戏可以重新开始
        app_state.game_state_manager.remove_game_state(game_id);

        // 清理该游戏的连接管理器，确保后续连接能够重新建立
        app_state
            .global_connection_manager
            .remove_game_manager(game_id.to_string())
            .await;

        // 清除回退游戏的全部日志和击杀记录，避免历史数据影响重新开始
        app_state
            .game_log_service
            .delete_logs_after_timestamp(game_id, None)
            .await
            .map_err(|e| DirectorError::OtherError {
                message: format!("Failed to clear game logs: {}", e),
            })?;

        app_state
            .game_log_service
            .delete_kill_records_after_timestamp(game_id, None)
            .await
            .map_err(|e| DirectorError::OtherError {
                message: format!("Failed to clear kill records: {}", e),
            })?;

        Ok(())
    }

    /// 结束游戏（进行中 → 结束）
    pub async fn end_game(&self, app_state: &AppState, game_id: &str) -> Result<(), DirectorError> {
        // 更新数据库中游戏状态为 "ended"
        let result = sqlx::query!(
            "UPDATE games SET status = 'ended', updated_at = CURRENT_TIMESTAMP WHERE id = ?",
            game_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| DirectorError::DatabaseError(e))?;

        if result.rows_affected() == 0 {
            return Err(DirectorError::GameNotFound);
        }

        // 断开所有连接（调用全局连接管理器的实现）
        app_state
            .global_connection_manager
            .remove_game_manager(game_id.to_string())
            .await;

        // 将当前游戏状态序列化并保存到磁盘文件
        app_state
            .game_state_manager
            .save_game_state_to_disk(game_id)
            .await
            .map_err(|e| DirectorError::OtherError {
                message: format!("Failed to save game state to disk: {}", e),
            })?;

        // 游戏已结束，清除内存中的游戏状态
        app_state.game_state_manager.remove_game_state(game_id);

        Ok(())
    }

    /// 恢复游戏（暂停 → 进行中）
    pub async fn resume_game(
        &self,
        app_state: &AppState,
        game_id: &str,
        save_file_name: Option<String>,
    ) -> Result<(), DirectorError> {
        // 检查是否提供了存档文件名
        let file_name = match save_file_name {
            Some(name) => name,
            None => {
                return Err(DirectorError::OtherError {
                    message: "必须提供存档文件名".to_string(),
                });
            }
        };

        // 更新数据库中游戏状态为 "running"
        let result = sqlx::query!(
            "UPDATE games SET status = 'running', updated_at = CURRENT_TIMESTAMP WHERE id = ?",
            game_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| DirectorError::DatabaseError(e))?;

        if result.rows_affected() == 0 {
            return Err(DirectorError::GameNotFound);
        }

        // 从指定的存档文件中恢复游戏状态
        app_state
            .game_state_manager
            .load_game_state_from_disk_with_name(game_id, &file_name)
            .await
            .map_err(|e| DirectorError::OtherError {
                message: format!("Failed to load game state from disk: {}", e),
            })?;

        // 获取恢复的游戏状态的保存时间
        if let Ok(game_state) = app_state.game_state_manager.get_game_state(game_id).await {
            let game_state_guard = game_state.read().await;
            if let Some(save_time) = game_state_guard.save_time {
                // 删除晚于保存时间的日志记录
                let _ = app_state
                    .game_log_service
                    .delete_logs_after_timestamp(game_id, Some(save_time))
                    .await;

                // 删除晚于保存时间的击杀记录
                let _ = app_state
                    .game_log_service
                    .delete_kill_records_after_timestamp(game_id, Some(save_time))
                    .await;
            }
        }

        Ok(())
    }

    /// 手动存盘操作
    pub async fn manual_save(
        &self,
        app_state: &AppState,
        game_id: &str,
        password: &str,
    ) -> Result<String, DirectorError> {
        // 验证导演密码
        self.verify_director_password(game_id, password).await?;

        // 执行存盘操作
        let save_file_name = app_state
            .game_state_manager
            .save_game_state_to_disk(game_id)
            .await
            .map_err(|e| DirectorError::OtherError {
                message: format!("Failed to save game state to disk: {}", e),
            })?;

        Ok(save_file_name)
    }

    /// 编辑游戏（导演端）
    pub async fn edit_game(
        &self,
        app_state: &AppState,
        game_id: &str,
        password: &str,
        request: DirectorEditGameRequest,
    ) -> Result<GameWithPlayerCounts, DirectorError> {
        // 1. 验证导演密码
        self.verify_director_password(game_id, password).await?;

        // 2. 验证请求参数
        request
            .validate()
            .map_err(|e| DirectorError::ValidationError { message: e })?;

        // 3. 执行字段更新操作
        if let Some(ref name) = request.name {
            sqlx::query("UPDATE games SET name = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?")
                .bind(name)
                .bind(game_id)
                .execute(&self.pool)
                .await?;
        }
        if let Some(ref description) = request.description {
            sqlx::query(
                "UPDATE games SET description = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
            )
            .bind(description)
            .bind(game_id)
            .execute(&self.pool)
            .await?;
        }
        if let Some(max_players) = request.max_players {
            sqlx::query(
                "UPDATE games SET max_players = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
            )
            .bind(max_players)
            .bind(game_id)
            .execute(&self.pool)
            .await?;
        }
        if let Some(ref rules_config) = request.rules_config {
            sqlx::query(
                "UPDATE games SET rules_config = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
            )
            .bind(rules_config)
            .bind(game_id)
            .execute(&self.pool)
            .await?;
        }

        // 4. 查询并返回更新后的游戏信息
        let game = app_state
            .game_service
            .get_game_by_id_with_player_counts(game_id, true)
            .await
            .map_err(|e| DirectorError::OtherError {
                message: format!("Failed to get game: {}", e),
            })?;

        Ok(game)
    }

    /// 查询存档文件列表
    pub async fn list_save_files(
        &self,
        app_state: &AppState,
        game_id: &str,
        password: &str,
    ) -> Result<Vec<SaveFileInfo>, DirectorError> {
        // 验证导演密码
        self.verify_director_password(game_id, password).await?;

        // 获取存档文件列表
        let save_files = app_state
            .game_state_manager
            .list_save_files(game_id)
            .await
            .map_err(|e| DirectorError::OtherError {
                message: format!("Failed to list save files: {}", e),
            })?;

        Ok(save_files)
    }
}
