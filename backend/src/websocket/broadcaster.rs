//! 新的WebSocket消息广播器
//! 负责向玩家和导演广播游戏状态更新消息，提供隐私保护机制

use crate::websocket::game_connection_manager::GameConnectionManager;
use crate::websocket::models::ActionResult;
use crate::websocket::models::{GameState, Player};
use serde_json::{Value as JsonValue, json};

/// 消息广播器
#[derive(Clone)]
pub struct MessageBroadcaster {
    /// 连接管理器
    connection_manager: GameConnectionManager,
}

impl MessageBroadcaster {
    /// 创建新的消息广播器
    pub fn new(connection_manager: GameConnectionManager) -> Self {
        Self { connection_manager }
    }

    /// 私有函数 - 生成导演视角消息
    pub fn generate_director_message(
        game_state: &GameState,
        action_result: Option<&ActionResult>,
    ) -> JsonValue {
        json!({
            "global_state": game_state.generate_global_state_info(),
            "game_data": {
                "players": game_state.players,
                "places": game_state.places,
            },
            "action_result": action_result.map(|res| res.to_client_response())
        })
    }

    /// 私有函数 - 生成玩家视角消息
    pub fn generate_player_message(
        game_state: &GameState,
        player: &Player,
        action_result: Option<&ActionResult>,
    ) -> JsonValue {
        // 构建玩家视角的地点信息（不包含其他玩家信息和物品信息）
        let actor_places: Vec<JsonValue> = game_state
            .places
            .values()
            .map(|place| {
                json!({
                    "name": place.name,
                    "is_destroyed": place.is_destroyed
                    // 注意：不包含players和items字段以保护玩家隐私
                })
            })
            .collect();

        // 构建玩家视角的玩家列表信息（不包括玩家id和名字以外的任何信息）
        let actor_players: Vec<JsonValue> = game_state
            .players
            .values()
            .map(|p| {
                json!({
                    "id": p.id,
                    "name": p.name
                    // 注意：不包含生命值、体力值、位置等敏感信息
                })
            })
            .collect();

        json!({
            "global_state": game_state.generate_global_state_info(),
            "game_data": {
                "player": player,
                "actor_players": actor_players,
                "actor_places": actor_places,
            },
            "action_result": action_result.map(|res| res.to_client_response())
        })
    }

    /// 公有函数 - 向所有导演广播
    pub async fn broadcast_to_directors(
        &self,
        game_state: &GameState,
        action_result: &ActionResult,
    ) -> Result<(), String> {
        let message =
            MessageBroadcaster::generate_director_message(game_state, Some(action_result));
        self.connection_manager
            .broadcast_to_directors(message)
            .await
    }

    /// 公有函数 - 向特定玩家广播
    pub async fn broadcast_to_players(
        &self,
        game_state: &GameState,
        player_ids: &[String],
        action_result: &ActionResult,
    ) -> Result<(), String> {
        for player_id in player_ids {
            // 获取玩家信息
            if let Some(player) = game_state.players.get(player_id) {
                let message = MessageBroadcaster::generate_player_message(
                    game_state,
                    player,
                    Some(action_result),
                );
                self.connection_manager
                    .broadcast_to_player(player_id, message)
                    .await?;
            }
        }
        Ok(())
    }
}
