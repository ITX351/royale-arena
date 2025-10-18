//! 游戏日志服务
//! 负责处理游戏日志的数据库操作

use crate::game::errors::GameError;
use crate::game::models::{
    GetPlayerMessagesRequest,
    MessageRecord,
    MessageType,
    KillRecord,
    GetPlayerKillRecordsRequest,
    NewKillRecord,
};
use chrono::{DateTime, Utc};
use sqlx::MySqlPool;
use uuid::Uuid;

/// 游戏日志服务
#[derive(Clone)]
pub struct GameLogService {
    pool: MySqlPool,
}

impl GameLogService {
    /// 创建新的游戏日志服务
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    /// 新增击杀记录
    pub async fn add_kill_record(
        &self,
        params: &NewKillRecord,
    ) -> Result<KillRecord, String> {
        let id = Uuid::new_v4().to_string();

        let killer_id_for_query = params.killer_id.as_deref();
        let weapon_for_query = params.weapon.as_deref();
        let location_for_query = params.location.as_deref();

        sqlx::query!(
            r#"
            INSERT INTO kill_records (id, game_id, killer_id, victim_id, kill_time, cause, weapon, location)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            id,
            &params.game_id,
            killer_id_for_query,
            &params.victim_id,
            params.kill_time,
            &params.cause,
            weapon_for_query,
            location_for_query
        )
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to create kill record: {}", e))?;

        Ok(KillRecord {
            id,
            game_id: params.game_id.clone(),
            killer_id: params.killer_id.clone(),
            victim_id: params.victim_id.clone(),
            kill_time: params.kill_time,
            cause: params.cause.clone(),
            weapon: params.weapon.clone(),
            location: params.location.clone(),
        })
    }

    /// 创建游戏日志
    pub async fn create_log(
        &self,
        game_id: &str,
        player_id: Option<String>,
        message: &str,
        message_type: MessageType,
        timestamp: DateTime<Utc>,
        visible_to_all_players: bool,
        visible_to_director: bool,
    ) -> Result<MessageRecord, String> {
        let id = Uuid::new_v4().to_string();

        // 根据消息类型确定数据库中的类型字符串
        let type_string = message_type.as_str();

        let player_id_for_query = player_id.as_deref();

        let result = sqlx::query!(
            r#"
            INSERT INTO game_logs (id, game_id, type, message, player_id, timestamp, visible_to_all_players, visible_to_director)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            id,
            game_id,
            type_string,
            message,
            player_id_for_query,
            timestamp,
            visible_to_all_players,
            visible_to_director
        )
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to create log: {}", e))?;

        if result.rows_affected() == 0 {
            return Err("Failed to insert log record".to_string());
        }

        Ok(MessageRecord {
            id,
            game_id: game_id.to_string(),
            message_type,
            message: message.to_string(),
            player_id,
            timestamp,
            visible_to_all_players,
            visible_to_director,
        })
    }

    /// 获取玩家消息记录
    pub async fn get_player_messages(
        &self,
        game_id: &str,
        player_id: &str,
        password: &str,
    ) -> Result<Vec<MessageRecord>, GameError> {
        // 验证请求参数
        let request = GetPlayerMessagesRequest {
            password: password.to_string(),
        };
        request.validate().map_err(GameError::ValidationError)?;

        // 验证玩家是否存在且密码正确
        let actor = sqlx::query!(
            "SELECT id FROM actors WHERE id = ? AND game_id = ? AND password = ?",
            player_id,
            game_id,
            password
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(GameError::DatabaseError)?;

        if actor.is_none() {
            return Err(GameError::ValidationError(
                "Invalid player credentials".to_string(),
            ));
        }

        // 查询玩家相关的消息记录，包括所有标记为visible_to_all_players为true的记录
        let messages = sqlx::query_as!(
            MessageRecord,
            r#"
            SELECT id, game_id, type as "message_type: MessageType", message, player_id, timestamp, 
                visible_to_all_players as "visible_to_all_players: bool",
                visible_to_director as "visible_to_director: bool"
            FROM game_logs 
            WHERE game_id = ? AND (player_id = ? OR visible_to_all_players = TRUE)
            ORDER BY timestamp ASC
            "#,
            game_id,
            player_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(GameError::DatabaseError)?;

        Ok(messages)
    }

    /// 获取导演消息记录
    pub async fn get_director_messages(
        &self,
        game_id: &str,
        password: &str,
    ) -> Result<Vec<MessageRecord>, GameError> {
        // 验证导演密码
        let game = sqlx::query!(
            "SELECT id FROM games WHERE id = ? AND director_password = ?",
            game_id,
            password
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(GameError::DatabaseError)?;

        if game.is_none() {
            return Err(GameError::ValidationError(
                "Invalid director credentials".to_string(),
            ));
        }

        // 查询所有标记为visible_to_director为true的记录
        let messages = sqlx::query_as!(
            MessageRecord,
            r#"
            SELECT id, game_id, type as "message_type: MessageType", message, player_id, timestamp, 
                visible_to_all_players as "visible_to_all_players: bool",
                visible_to_director as "visible_to_director: bool"
            FROM game_logs 
            WHERE game_id = ? AND visible_to_director = TRUE
            ORDER BY timestamp ASC
            "#,
            game_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(GameError::DatabaseError)?;

        Ok(messages)
    }

    /// 删除指定时间戳之后的日志记录
    pub async fn delete_logs_after_timestamp(
        &self,
        game_id: &str,
        timestamp: Option<DateTime<Utc>>,
    ) -> Result<u64, GameError> {
        let rows_affected = if let Some(ts) = timestamp {
            sqlx::query!(
                "DELETE FROM game_logs WHERE game_id = ? AND timestamp > ?",
                game_id,
                ts
            )
            .execute(&self.pool)
            .await
        } else {
            sqlx::query!("DELETE FROM game_logs WHERE game_id = ?", game_id)
                .execute(&self.pool)
                .await
        }
        .map_err(GameError::DatabaseError)?
        .rows_affected();

        Ok(rows_affected)
    }

    /// 获取玩家击杀记录
    pub async fn get_player_kill_records(
        &self,
        game_id: &str,
        player_id: &str,
        password: &str,
    ) -> Result<Vec<KillRecord>, GameError> {
        // 验证请求参数
        let request = GetPlayerKillRecordsRequest {
            password: password.to_string(),
        };

        // 验证玩家是否存在且密码正确
        let actor = sqlx::query!(
            "SELECT id FROM actors WHERE id = ? AND game_id = ? AND password = ?",
            player_id,
            game_id,
            password
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(GameError::DatabaseError)?;

        if actor.is_none() {
            return Err(GameError::ValidationError(
                "Invalid player credentials".to_string(),
            ));
        }

        // 查询玩家相关的击杀记录（作为击杀者）
        let kill_records = sqlx::query_as!(
            KillRecord,
            r#"
            SELECT id, game_id, killer_id, victim_id, kill_time, cause, weapon, location
            FROM kill_records 
            WHERE game_id = ? AND killer_id = ?
            ORDER BY kill_time ASC
            "#,
            game_id,
            player_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(GameError::DatabaseError)?;

        Ok(kill_records)
    }

    /// 获取导演击杀记录
    pub async fn get_director_kill_records(
        &self,
        game_id: &str,
        password: &str,
    ) -> Result<Vec<KillRecord>, GameError> {
        // 验证导演密码
        let game = sqlx::query!(
            "SELECT id FROM games WHERE id = ? AND director_password = ?",
            game_id,
            password
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(GameError::DatabaseError)?;

        if game.is_none() {
            return Err(GameError::ValidationError(
                "Invalid director credentials".to_string(),
            ));
        }

        // 查询所有击杀记录
        let kill_records = sqlx::query_as!(
            KillRecord,
            r#"
            SELECT id, game_id, killer_id, victim_id, kill_time, cause, weapon, location
            FROM kill_records 
            WHERE game_id = ?
            ORDER BY kill_time ASC
            "#,
            game_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(GameError::DatabaseError)?;

        Ok(kill_records)
    }

    /// 删除指定时间戳之后的击杀记录
    pub async fn delete_kill_records_after_timestamp(
        &self,
        game_id: &str,
        timestamp: Option<DateTime<Utc>>,
    ) -> Result<u64, GameError> {
        let rows_affected = if let Some(ts) = timestamp {
            sqlx::query!(
                "DELETE FROM kill_records WHERE game_id = ? AND kill_time > ?",
                game_id,
                ts
            )
            .execute(&self.pool)
            .await
        } else {
            sqlx::query!("DELETE FROM kill_records WHERE game_id = ?", game_id)
                .execute(&self.pool)
                .await
        }
        .map_err(GameError::DatabaseError)?
        .rows_affected();

        Ok(rows_affected)
    }
}