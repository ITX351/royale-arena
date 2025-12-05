//! 新的WebSocket消息广播器
//! 负责向玩家和导演广播游戏状态更新消息，提供隐私保护机制

use crate::websocket::game_connection_manager::GameConnectionManager;
use crate::websocket::models::SearchResultType;
use crate::websocket::models::{ActionResult, GameState, Place, Player};
use chrono::Utc;
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
            "global_state": game_state.to_director_client_json(),
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
            .map(|place| place.to_player_client_json())
            .collect();

        // 构建玩家视角的玩家列表信息（不包括玩家id和名字以外的任何信息）
        let actor_players: Vec<JsonValue> = game_state
            .players
            .values()
            .map(|p| p.to_player_client_json_for_other_players())
            .collect();

        json!({
            "global_state": game_state.to_player_client_json(),
            "game_data": {
                "player": player.to_player_client_clone_for_self(), // 使用处理过的Player实例，last_search_result被设置为None
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

// 广播相关的JSON转换函数
impl Player {
    /// 生成用于玩家客户端的Player副本（对last_search_result进行隐私处理）
    pub fn to_player_client_clone_for_self(&self) -> Self {
        let mut player = self.clone();
        // 如果搜索结果不可见，则脱敏处理（移除目标名称和ID，无论目标是玩家还是物品）
        if let Some(ref mut search_result) = player.last_search_result {
            if !search_result.is_visible {
                match search_result.target_type {
                    SearchResultType::Player => {
                        search_result.target_name = String::from("未知玩家");
                        search_result.target_id = String::from("");
                    }
                    SearchResultType::Item => {
                        search_result.target_name = String::from("未知物品");
                        search_result.target_id = String::from("");
                    }
                }
            }
        }
        player.bleed_inflictor = None; // 移除流血附加者信息
        player
    }

    pub fn to_player_client_json_for_other_players(&self) -> JsonValue {
        json!({
            "id": self.id,
            "name": self.name,
        })
    }
}

impl Place {
    pub fn to_player_client_json(&self) -> JsonValue {
        json!({
            "name": self.name,
            "is_destroyed": self.is_destroyed,
        })
    }
}

impl GameState {
    /// 生成导演视角的全局状态信息
    pub fn to_director_client_json(&self) -> JsonValue {
        json!({
            "weather": self.weather,
            "night_start_time": self.night_start_time,
            "night_end_time": self.night_end_time,
            "next_night_destroyed_places": self.next_night_destroyed_places,
            "rules_config": self.rules_config,
            "server_now": Utc::now(),
        })
    }

    /// 生成玩家视角的全局状态信息
    pub fn to_player_client_json(&self) -> JsonValue {
        json!({
            "weather": self.weather,
            "night_start_time": self.night_start_time,
            "night_end_time": self.night_end_time,
            "next_night_destroyed_places": self.next_night_destroyed_places,
            "rules_config": self.rules_config,
            "server_now": Utc::now(),
        })
    }
}

impl ActionResult {
    /// 创建用于返回给前端的数据结构，排除`broadcast_players`字段
    pub fn to_client_response(&self) -> JsonValue {
        json!({
            "data": self.data,
            "log_message": self.log_message,
            "message_type": self.message_type,
            "timestamp": self.timestamp
        })
    }
}
