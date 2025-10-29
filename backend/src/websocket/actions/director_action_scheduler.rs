//! 导演行动调度器
//!
//! 负责导演行动的分发调度，导演拥有特殊权限，无需验证前置条件

use crate::websocket::models::{ActionResults, AirdropItem, GameState, ItemDeletionItem};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

/// 导演行动参数结构
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DirectorActionParams {
    /// 时间设置
    pub timestamp: Option<String>,

    /// 地点操作
    pub place_name: Option<String>,
    pub is_destroyed: Option<bool>,
    pub places: Option<Vec<JsonValue>>,

    /// 天气操作
    pub weather: Option<f64>,

    /// 玩家操作
    pub player_id: Option<String>,
    pub life: Option<i32>,     // 玩家生命值
    pub strength: Option<i32>, // 玩家体力值
    pub target_place: Option<String>,
    pub action_type: Option<String>, // rope/unrope

    /// 道具操作
    pub target_type: Option<String>,
    pub item_name: Option<String>, // 物品名称，用于添加/移除玩家物品

    /// 消息操作
    pub message: Option<String>,

    /// 批量操作
    pub airdrops: Option<Vec<AirdropItem>>,
    pub deletions: Option<Vec<ItemDeletionItem>>,
    pub clear_all: Option<bool>,
}

impl DirectorActionParams {
    /// 从JSON数据构造DirectorActionParams
    pub fn from_json(data: &JsonValue) -> Result<Self, String> {
        serde_json::from_value(data.clone())
            .map_err(|e| format!("Failed to parse director action params: {}", e))
    }
}

/// 导演行动调度器
pub struct DirectorActionScheduler;

impl DirectorActionScheduler {
    /// 调度导演行动
    ///
    /// # 参数
    /// - `game_state`: 可变游戏状态引用
    /// - `action_type`: 导演行动类型
    /// - `action_params`: 导演行动参数
    ///
    /// # 返回值
    /// - `Ok(ActionResults)`: 行动执行成功或失败
    /// - `Err(String)`: 系统级错误
    pub fn dispatch(
        game_state: &mut GameState,
        action_type: &str,
        action_params: DirectorActionParams,
    ) -> Result<ActionResults, String> {
        match action_type {
            "set_night_start_time" => {
                let timestamp = action_params
                    .timestamp
                    .ok_or_else(|| "Missing timestamp parameter".to_string())?;
                game_state.handle_set_night_start_time(&timestamp)
            }

            "set_night_end_time" => {
                let timestamp = action_params
                    .timestamp
                    .ok_or_else(|| "Missing timestamp parameter".to_string())?;
                game_state.handle_set_night_end_time(&timestamp)
            }

            "modify_place" => {
                let place_name = action_params
                    .place_name
                    .ok_or_else(|| "Missing place_name parameter".to_string())?;
                let is_destroyed = action_params
                    .is_destroyed
                    .ok_or_else(|| "Missing is_destroyed parameter".to_string())?;
                game_state.handle_modify_place(&place_name, is_destroyed)
            }

            "set_destroy_places" => {
                let places = action_params
                    .places
                    .ok_or_else(|| "Missing places parameter".to_string())?;
                game_state.handle_set_destroy_places(&places)
            }

            "batch_airdrop" => {
                let airdrops = action_params
                    .airdrops
                    .ok_or_else(|| "Missing airdrops parameter".to_string())?;
                game_state.handle_batch_airdrop(airdrops)
            }

            "batch_item_deletion" => {
                let deletions = action_params.deletions.unwrap_or_default();
                let clear_all = action_params.clear_all.unwrap_or(false);
                game_state.handle_batch_item_deletion(deletions, clear_all)
            }

            "weather" => {
                let weather = action_params
                    .weather
                    .ok_or_else(|| "Missing weather parameter".to_string())?;
                game_state.handle_weather(weather)
            }

            "life" => {
                let player_id = action_params
                    .player_id
                    .ok_or_else(|| "Missing player_id parameter".to_string())?;
                let life = action_params
                    .life
                    .ok_or_else(|| "Missing life parameter".to_string())?;
                game_state.handle_set_player_life(&player_id, life)
            }

            "strength" => {
                let player_id = action_params
                    .player_id
                    .ok_or_else(|| "Missing player_id parameter".to_string())?;
                let strength = action_params
                    .strength
                    .ok_or_else(|| "Missing strength parameter".to_string())?;
                game_state.handle_set_player_strength(&player_id, strength)
            }

            "add_player_item" => {
                let player_id = action_params
                    .player_id
                    .ok_or_else(|| "Missing player_id parameter".to_string())?;
                let item_name = action_params
                    .item_name
                    .ok_or_else(|| "Missing item_name parameter".to_string())?;
                game_state.handle_add_player_item(&player_id, &item_name)
            }

            "remove_player_item" => {
                let player_id = action_params
                    .player_id
                    .ok_or_else(|| "Missing player_id parameter".to_string())?;
                let item_name = action_params
                    .item_name
                    .ok_or_else(|| "Missing item_name parameter".to_string())?;
                game_state.handle_remove_player_item(&player_id, &item_name)
            }

            "rope" | "unrope" => {
                let player_id = action_params
                    .player_id
                    .ok_or_else(|| "Missing player_id parameter".to_string())?;
                game_state.handle_rope_action(&player_id, action_type)
            }

            "broadcast" => {
                let message = action_params
                    .message
                    .ok_or_else(|| "Missing message parameter".to_string())?;
                game_state.handle_broadcast(&message)
            }

            "message_to_player" => {
                let player_id = action_params
                    .player_id
                    .ok_or_else(|| "Missing player_id parameter".to_string())?;
                let message = action_params
                    .message
                    .ok_or_else(|| "Missing message parameter".to_string())?;
                game_state.handle_director_message_to_player(&player_id, &message)
            }

            "night_settlement" => game_state.handle_night_settlement(),

            _ => Err(format!("Unknown director action type: {}", action_type)),
        }
    }
}
