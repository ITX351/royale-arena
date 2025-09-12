use sqlx::{MySqlPool, Row};
use uuid::Uuid;
use crate::director::{DirectorError, models::*};

/// 导演服务层
#[derive(Clone)]
pub struct DirectorService {
    pool: MySqlPool,
}

impl DirectorService {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    /// 验证导演密码
    pub async fn verify_director_password(
        &self, 
        game_id: &str, 
        password: &str
    ) -> Result<(), DirectorError> {
        let row = sqlx::query(
            "SELECT director_password FROM games WHERE id = ?"
        )
        .bind(game_id)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => {
                let stored_password: String = row.get("director_password");
                if stored_password == password {
                    Ok(())
                } else {
                    Err(DirectorError::InvalidDirectorPassword)
                }
            }
            None => Err(DirectorError::GameNotFound),
        }
    }

    /// 检查游戏状态是否允许删除演员
    async fn check_game_status_for_deletion(&self, game_id: &str) -> Result<(), DirectorError> {
        let row = sqlx::query(
            "SELECT status FROM games WHERE id = ?"
        )
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
    async fn check_player_name_exists(&self, game_id: &str, name: &str) -> Result<bool, DirectorError> {
        let row = sqlx::query(
            "SELECT COUNT(*) as count FROM actors WHERE game_id = ? AND name = ?"
        )
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
             VALUES (?, ?, ?, ?, ?, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)"
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
            match self.check_player_name_exists(game_id, &player_request.player_name).await {
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
            match self.create_player(
                game_id,
                &player_request.player_name,
                &player_request.password,
                player_request.get_team_id(),
            ).await {
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
             ORDER BY created_at ASC"
        )
        .bind(game_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(players)
    }

    /// 获取演员信息（内部方法）
    async fn get_player_by_id(&self, player_id: &str) -> Result<PlayerInfo, DirectorError> {
        let player = sqlx::query_as::<_, PlayerInfo>(
            "SELECT id, game_id, name, password, team_id FROM actors WHERE id = ?"
        )
        .bind(player_id)
        .fetch_optional(&self.pool)
        .await?;

        match player {
            Some(player) => Ok(player),
            None => Err(DirectorError::PlayerNotFound { id: player_id.to_string() }),
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
            return Err(DirectorError::PlayerNotFound { id: player_id.to_string() });
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
}