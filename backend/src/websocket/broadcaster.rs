//! 新的WebSocket消息广播器
//! 负责向玩家和导演广播游戏状态更新消息，提供隐私保护机制

use serde_json::{json, Value as JsonValue};
use crate::websocket::connection_manager::ConnectionManager;
use crate::websocket::models::{GameState, Player};
use crate::websocket::models::ActionResult;

/// 消息广播器
#[derive(Clone)]
pub struct MessageBroadcaster {
    /// 连接管理器
    connection_manager: ConnectionManager,
}

impl MessageBroadcaster {
    /// 创建新的消息广播器
    pub fn new(connection_manager: ConnectionManager) -> Self {
        Self {
            connection_manager,
        }
    }

    /// 私有函数 - 生成导演视角消息
    fn generate_director_message(&self, game_state: &GameState, action_result: &ActionResult) -> JsonValue {
        // 构建导演视角的地点信息（包含所有玩家信息）
        let places: Vec<JsonValue> = game_state.places.values().map(|place| {
            json!({
                "name": place.name,
                "players": place.players,
                "items": place.items,
                "is_destroyed": place.is_destroyed
            })
        }).collect();

        json!({
            "global_state": {
                "game_phase": game_state.game_phase,
                "weather": game_state.weather,
                "night_start_time": game_state.night_start_time,
                "night_end_time": game_state.night_end_time,
                "next_night_destroyed_places": game_state.next_night_destroyed_places,
                "players": game_state.players,
                "places": places
            },
            "action_result": action_result.to_client_response()
        })
    }

    /// 私有函数 - 生成玩家视角消息
    fn generate_player_message(&self, game_state: &GameState, player: &Player, action_result: &ActionResult) -> JsonValue {
        // 构建玩家视角的地点信息（不包含其他玩家信息和物品信息）
        let places: Vec<JsonValue> = game_state.places.values().map(|place| {
            json!({
                "name": place.name,
                "is_destroyed": place.is_destroyed
                // 注意：不包含players和items字段以保护玩家隐私
            })
        }).collect();

        json!({
            "global_state": {
                "game_phase": game_state.game_phase,
                "weather": game_state.weather,
                "night_start_time": game_state.night_start_time,
                "night_end_time": game_state.night_end_time,
                "next_night_destroyed_places": game_state.next_night_destroyed_places,
                "player": {
                    "id": player.id,
                    "name": player.name,
                    "location": player.location,
                    "life": player.life,
                    "strength": player.strength,
                    "is_alive": player.is_alive,
                    "rest_mode": player.rest_mode
                },
                "places": places
            },
            "action_result": action_result.to_client_response()
        })
    }

    /// 公有函数 - 向所有导演广播
    pub async fn broadcast_to_directors(&self, game_state: &GameState, action_result: &ActionResult) -> Result<(), String> {
        let message = self.generate_director_message(game_state, action_result);
        self.connection_manager.broadcast_to_directors(message).await
    }

    /// 公有函数 - 向特定玩家广播
    pub async fn broadcast_to_players(&self, game_state: &GameState, player_ids: &[String], action_result: &ActionResult) -> Result<(), String> {
        for player_id in player_ids {
            // 获取玩家信息
            if let Some(player) = game_state.players.get(player_id) {
                let message = self.generate_player_message(game_state, player, action_result);
                self.connection_manager.broadcast_to_player(player_id, message).await?;
            }
        }
        Ok(())
    }
}