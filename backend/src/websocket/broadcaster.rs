//! 新的WebSocket消息广播器
//! 负责向玩家和导演广播游戏状态更新消息，提供隐私保护机制

use crate::websocket::game_connection_manager::GameConnectionManager;
use crate::websocket::models::SearchResultType;
use crate::websocket::models::{ActionResult, GameState, Place, Player};
use chrono::Utc;
use serde_json::{Value as JsonValue, json};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy, PartialEq, Eq)]
struct TeammateViewFields {
    team_id: Option<u32>,
    is_alive: bool,
    life: i32,
    strength: i32,
    item_count: usize,
    max_backpack_items: usize,
}

impl From<&Player> for TeammateViewFields {
    fn from(player: &Player) -> Self {
        Self {
            team_id: player.team_id,
            is_alive: player.is_alive,
            life: player.life,
            strength: player.strength,
            item_count: player.get_total_item_count(),
            max_backpack_items: player.max_backpack_items,
        }
    }
}

pub(crate) struct TeammateViewSnapshot {
    mode: i32,
    night_start_time: Option<chrono::DateTime<Utc>>,
    night_end_time: Option<chrono::DateTime<Utc>>,
    players: HashMap<String, TeammateViewFields>,
}

impl TeammateViewSnapshot {
    pub(crate) fn capture(game_state: &GameState) -> Self {
        Self {
            mode: game_state.rule_engine.teammate_behavior.mode,
            night_start_time: game_state.night_start_time,
            night_end_time: game_state.night_end_time,
            players: game_state
                .players
                .iter()
                .map(|(id, player)| (id.clone(), player.into()))
                .collect(),
        }
    }

    pub(crate) fn recipients_after_change(&self, game_state: &GameState) -> Vec<String> {
        if self.mode != game_state.rule_engine.teammate_behavior.mode
            || self.night_start_time != game_state.night_start_time
            || self.night_end_time != game_state.night_end_time
        {
            return game_state.players.keys().cloned().collect();
        }

        let mut affected_teams = HashSet::new();
        let mut affected_players = HashSet::new();
        for (id, player) in &game_state.players {
            let current = TeammateViewFields::from(player);
            let previous = self.players.get(id);
            if previous != Some(&current) {
                affected_players.insert(id.clone());
                for team_id in [previous.and_then(|p| p.team_id), current.team_id] {
                    if let Some(team_id) = team_id.filter(|id| *id > 0) {
                        affected_teams.insert(team_id);
                    }
                }
            }
        }
        for (id, previous) in &self.players {
            if !game_state.players.contains_key(id) {
                if let Some(team_id) = previous.team_id.filter(|id| *id > 0) {
                    affected_teams.insert(team_id);
                }
            }
        }
        game_state
            .players
            .iter()
            .filter(|(id, player)| {
                affected_players.contains(*id)
                    || player
                        .team_id
                        .is_some_and(|id| affected_teams.contains(&id))
            })
            .map(|(id, _)| id.clone())
            .collect()
    }
}

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
            .map(|p| p.to_player_client_json_for_other_players(player, game_state))
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

    /// 只刷新状态，不产生新的行动消息或日志。
    pub async fn broadcast_player_snapshots(
        &self,
        game_state: &GameState,
        player_ids: &[String],
    ) -> Result<(), String> {
        for player_id in player_ids {
            if let Some(player) = game_state.players.get(player_id) {
                let message = Self::generate_player_message(game_state, player, None);
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

    pub fn to_player_client_json_for_other_players(
        &self,
        viewer: &Player,
        game_state: &GameState,
    ) -> JsonValue {
        let is_teammate =
            self.team_id.is_some_and(|team_id| team_id > 0) && self.team_id == viewer.team_id;
        let show_status =
            is_teammate && game_state.rule_engine.teammate_behavior.is_status_visible();
        let can_receive_transfer = is_teammate
            && game_state
                .rule_engine
                .teammate_behavior
                .is_transfer_enabled()
            && self.is_alive
            && self.strength >= 5
            && self.get_total_item_count() < self.max_backpack_items;
        json!({
            "id": self.id,
            "name": self.name,
            "team_id": if is_teammate { self.team_id } else { None },
            "is_alive": if show_status { Some(self.is_alive) } else { None },
            "life": if show_status { Some(self.life) } else { None },
            "strength": if show_status { Some(self.strength) } else { None },
            "can_receive_transfer": can_receive_transfer,
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
            "shop": self.shop,
            "sell_prices": self.sell_prices,
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
            "shop": self.shop,
            "sell_prices": self.sell_prices,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn teammate_snapshot_refreshes_eligible_viewers_after_status_changes() {
        let rules = json!({
            "map": {"places": ["loc"], "safe_places": []},
            "player": {"max_life": 100, "max_strength": 100, "daily_life_recovery": 0, "daily_strength_recovery": 40, "search_cooldown": 30, "max_backpack_items": 6, "unarmed_damage": 5},
            "action_costs": {"move": 5, "search": 5, "pick": 0, "attack": 0, "equip": 0, "use": 0, "throw": 0, "deliver": 10},
            "rest_mode": {"life_recovery": 25, "strength_recovery": 1000, "max_moves": 1},
            "teammate_behavior": 12,
            "items_config": {"rarity_levels": [], "items": {}, "upgrade_recipes": {}}
        });
        let mut state = GameState::new("game".to_string(), rules);
        for (id, team) in [("viewer", 1), ("target", 1), ("outsider", 2)] {
            state.players.insert(
                id.to_string(),
                Player::new(
                    id.to_string(),
                    id.to_string(),
                    "pw".to_string(),
                    team,
                    &state.rule_engine,
                ),
            );
        }

        let before = TeammateViewSnapshot::capture(&state);
        assert!(before.recipients_after_change(&state).is_empty());
        state.players.get_mut("target").unwrap().strength = 4;
        let recipients = before.recipients_after_change(&state);
        assert!(recipients.contains(&"viewer".to_string()));
        assert!(recipients.contains(&"target".to_string()));
        assert!(!recipients.contains(&"outsider".to_string()));
        let target = &state.players["target"];
        let viewer = &state.players["viewer"];
        let message = target.to_player_client_json_for_other_players(viewer, &state);
        assert_eq!(message["strength"].as_i64(), Some(4));
        assert_eq!(message["can_receive_transfer"].as_bool(), Some(false));

        let before = TeammateViewSnapshot::capture(&state);
        let target = state.players.get_mut("target").unwrap();
        target.strength = 100;
        target.max_backpack_items = 0;
        assert!(
            before
                .recipients_after_change(&state)
                .contains(&"viewer".to_string())
        );
        let message = state.players["target"]
            .to_player_client_json_for_other_players(&state.players["viewer"], &state);
        assert_eq!(message["can_receive_transfer"].as_bool(), Some(false));

        let before = TeammateViewSnapshot::capture(&state);
        state.rule_engine.teammate_behavior.mode = 0;
        assert_eq!(before.recipients_after_change(&state).len(), 3);

        let before = TeammateViewSnapshot::capture(&state);
        state.night_start_time = Some(Utc::now());
        assert_eq!(before.recipients_after_change(&state).len(), 3);
    }
}
