//! 游戏日志服务
//! 负责处理游戏日志的数据库操作

use sqlx::MySqlPool;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::game::models::{MessageRecord, MessageType, GetPlayerMessagesRequest};
use crate::game::errors::GameError;

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

    /// 创建游戏日志
    pub async fn create_log(
        &self,
        game_id: &str,
        player_id: &str,
        message: &str,
        message_type: MessageType,
        timestamp: DateTime<Utc>,  // 添加时间戳参数
    ) -> Result<MessageRecord, String> {
        let id = Uuid::new_v4().to_string();
        
        // 根据消息类型确定数据库中的类型字符串
        let type_string = message_type.as_str();

        let result = sqlx::query!(
            r#"
            INSERT INTO game_logs (id, game_id, type, message, player_id, timestamp)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
            id,
            game_id,
            type_string,
            message,
            player_id,
            timestamp  // 使用传入的时间戳
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
            player_id: player_id.to_string(),
            timestamp,  // 使用传入的时间戳
        })
    }

    /// 获取玩家消息记录
    pub async fn get_player_messages(&self, game_id: &str, player_id: &str, password: &str) -> Result<Vec<MessageRecord>, GameError> {
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
            return Err(GameError::ValidationError("Invalid player credentials".to_string()));
        }
        
        // 查询玩家相关的消息记录
        let messages = sqlx::query_as!(
            MessageRecord,
            r#"
            SELECT id, game_id, type as "message_type: MessageType", message, player_id, timestamp
            FROM game_logs 
            WHERE game_id = ? AND player_id = ?
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
}