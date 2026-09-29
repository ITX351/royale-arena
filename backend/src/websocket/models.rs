//! WebSocket相关模型定义

use crate::game::game_rule_engine::{GameRuleEngine, Item};
use crate::game::models::MessageType;
use crate::websocket::actions::utils::restore_item_nightly_uses;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value as JsonValue;
use std::collections::HashMap;

/// WebSocket连接类型
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum ConnectionType {
    /// 玩家连接
    #[serde(rename = "actor")]
    Actor,
    /// 导演连接
    #[serde(rename = "director")]
    Director,
}

/// WebSocket认证请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WebSocketAuthRequest {
    /// 用户类型
    pub user_type: ConnectionType,
    /// 密码
    pub password: String,
}

/// WebSocket消息类型
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum WebSocketMessageType {
    /// 玩家行动
    #[serde(rename = "player_action")]
    PlayerAction,
    /// 导演控制
    #[serde(rename = "director_action")]
    DirectorAction,
}

/// WebSocket客户端消息
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WebSocketClientMessage {
    /// 消息类型
    #[serde(rename = "type")]
    pub message_type: WebSocketMessageType,
    /// 消息数据
    pub data: JsonValue,
}

/// 批量空投请求项结构
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AirdropItem {
    pub item_name: String,
    pub place_name: String,
}

/// 批量物品删除请求项结构
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ItemDeletionItem {
    pub place_name: String,
    pub item_name: Option<String>, // None表示清空地点所有物品
}

/// 商店上架物品结构
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ShopListing {
    /// 上架条目唯一ID
    pub id: String,
    /// 物品名称（来自规则配置）
    pub item_name: String,
    /// 价格（货币数，单价）
    pub price: i32,
    /// 库存数量
    #[serde(default = "default_quantity")]
    pub quantity: i32,
    /// 稀有度随机条目：Some("weapon" | "armor")；具体物品条目为 None
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub item_kind: Option<String>,
    /// 与 item_kind 同时出现的稀有度（common | rare | epic | legendary）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rarity: Option<String>,
}

fn default_quantity() -> i32 {
    1
}

/// 售出系统稀有度价格条目
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SellPriceEntry {
    /// 条目唯一ID
    pub id: String,
    /// 稀有度：common | rare | epic | legendary
    pub rarity: String,
    /// 售出价格（货币数，0.5 步长）
    pub price: f64,
}

/// 玩家购买请求项
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ShopBuyItem {
    /// 商品上架ID
    pub listing_id: String,
    /// 购买数量
    pub quantity: i32,
}

// /// WebSocket服务端消息
// #[derive(Debug, Clone, Deserialize, Serialize)]
// pub struct WebSocketServerMessage {
//     /// 消息类型
//     #[serde(rename = "type")]
//     pub message_type: WebSocketMessageType,
//     /// 消息数据
//     pub data: JsonValue,
// }

/// 游戏状态类
#[derive(Debug, Clone, Serialize)]
pub struct GameState {
    /// 游戏ID
    pub game_id: String,
    /// 玩家状态映射，键为玩家ID
    pub players: HashMap<String, Player>,
    /// 地点状态映射，键为地点名称
    pub places: HashMap<String, Place>,
    /// 天气条件（影响搜索可见性）
    pub weather: f64,
    /// 投票记录，键为投票者ID，值为被投票者ID
    pub votes: HashMap<String, String>,
    /// 游戏规则配置（原始JSON）
    pub rules_config: JsonValue,
    /// 解析后的规则引擎（用于运行时规则查询）
    #[serde(skip)]
    pub rule_engine: GameRuleEngine,
    /// 夜晚开始时间
    pub night_start_time: Option<DateTime<Utc>>,
    /// 夜晚结束时间
    pub night_end_time: Option<DateTime<Utc>>,
    /// 下一夜晚缩圈地点集合
    pub next_night_destroyed_places: Vec<String>,
    /// 存档时间
    pub save_time: Option<DateTime<Utc>>,
    /// 商店上架物品列表
    #[serde(default)]
    pub shop: Vec<ShopListing>,
    /// 售出系统：稀有度 → 价格（同一稀有度最多一条）
    #[serde(default)]
    pub sell_prices: Vec<SellPriceEntry>,
}

/// 玩家类
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    /// 玩家ID
    pub id: String,
    /// 玩家名称
    pub name: String,
    /// 玩家密码
    pub password: String,
    /// 当前位置
    pub location: String,
    /// 当前生命值
    pub life: i32,
    /// 当前体力值
    pub strength: i32,
    /// 最大生命值（可能被规则或道具影响）
    pub max_life: i32,
    /// 最大体力值（可能被规则或道具影响）
    pub max_strength: i32,
    /// 物品背包
    pub inventory: Vec<Item>,
    /// 当前装备的武器（单槽位）
    pub equipped_weapon: Option<Item>,
    /// 当前装备的防具（单槽位）
    pub equipped_armor: Option<Item>,
    /// 上一次搜索结果
    pub last_search_result: Option<SearchResult>,
    /// 是否存活
    pub is_alive: bool,
    /// 是否被捆绑（禁止行动）
    pub is_bound: bool,
    /// 是否处于静养模式
    pub rest_mode: bool,
    /// 静养模式下的移动次数限制
    pub rest_moves_used: i32,
    /// 上次搜索时间
    pub last_search_time: Option<DateTime<Utc>>,
    /// 队伍ID（用于队友行为判断）
    pub team_id: Option<u32>,
    /// 持续伤害效果（流血状态）
    pub bleed_damage: i32,
    /// 附加流血的玩家ID
    #[serde(default)]
    pub bleed_inflictor: Option<String>,
    /// 背包容量上限（初始来自规则，可被永久增益道具提升）
    #[serde(default)]
    pub max_backpack_items: usize,
    /// 货币总数
    #[serde(default)]
    pub coins: f64,
}

impl Player {
    /// 创建新的玩家（使用规则引擎的默认值）
    pub fn new(
        id: String,
        name: String,
        password: String,
        team_id: u32,
        rule_engine: &GameRuleEngine,
    ) -> Self {
        let max_life = rule_engine.player_config.max_life;
        let max_strength = rule_engine.player_config.max_strength;
        let max_backpack_items = rule_engine.player_config.max_backpack_items;

        Self {
            id,
            name,
            password,
            location: String::new(),
            life: max_life,
            strength: max_strength,
            max_life,
            max_strength,
            max_backpack_items,
            inventory: Vec::new(),
            equipped_weapon: None,
            equipped_armor: None,
            last_search_result: None,
            is_alive: true,
            is_bound: false,
            rest_mode: true,
            rest_moves_used: 0,
            last_search_time: None,
            team_id: Some(team_id),
            bleed_damage: 0,
            bleed_inflictor: None,
            coins: 0.0,
        }
    }

    /// 计算总物品数量（背包 + 已装备武器 + 已装备防具）
    pub fn get_total_item_count(&self) -> usize {
        let mut count = self.inventory.len();
        if self.equipped_weapon.is_some() {
            count += 1;
        }
        if self.equipped_armor.is_some() {
            count += 1;
        }
        count
    }
    /// 设置持续伤害效果
    pub fn update_bleed_effect(&mut self, damage: i32, inflictor: Option<String>) -> bool {
        if damage >= self.bleed_damage {
            self.bleed_damage = damage;
            self.bleed_inflictor = inflictor;
            true
        } else {
            false
        }
    }

    /// 清除持续伤害效果
    pub fn clear_bleed_effect(&mut self) {
        self.bleed_damage = 0;
        self.bleed_inflictor = None;
    }

    /// 检查是否有持续伤害效果
    pub fn has_bleed_effect(&self) -> bool {
        self.bleed_damage > 0
    }

    /// 装备武器（如已有装备则返回旧装备）
    pub fn equip_weapon(&mut self, weapon: Item) -> Option<Item> {
        self.equipped_weapon.replace(weapon)
    }

    /// 装备防具（如已有装备则返回旧装备）
    pub fn equip_armor(&mut self, armor: Item) -> Option<Item> {
        self.equipped_armor.replace(armor)
    }

    /// 卸下武器并返回
    pub fn unequip_weapon(&mut self) -> Option<Item> {
        self.equipped_weapon.take()
    }

    /// 卸下防具并返回
    pub fn unequip_armor(&mut self) -> Option<Item> {
        self.equipped_armor.take()
    }

    /// 每日清除玩家状态
    pub fn daily_reset(&mut self, rule_engine: &GameRuleEngine) {
        self.rest_mode = true;
        self.rest_moves_used = 0;
        self.last_search_result = None;
        self.is_bound = false;

        self.reset_nightly_uses(rule_engine);
    }

    fn reset_nightly_uses(&mut self, rule_engine: &GameRuleEngine) {
        for item in &mut self.inventory {
            restore_item_nightly_uses(item, rule_engine);
        }

        if let Some(weapon) = self.equipped_weapon.as_mut() {
            restore_item_nightly_uses(weapon, rule_engine);
        }

        if let Some(armor) = self.equipped_armor.as_mut() {
            restore_item_nightly_uses(armor, rule_engine);
        }
    }
}

/// 地点类
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Place {
    /// 地点名称
    pub name: String,
    /// 在该地点的玩家ID列表
    pub players: Vec<String>,
    /// 在该地点的物品列表
    pub items: Vec<Item>,
    /// 是否已被摧毁（缩圈）
    pub is_destroyed: bool,
}

impl Place {
    /// 创建新的地点
    pub fn new(name: String) -> Self {
        Self {
            name,
            players: Vec::new(),
            items: Vec::new(),
            is_destroyed: false,
        }
    }

    /// 恢复地点所有物品的夜晚使用次数
    pub fn reset_nightly_item_uses(&mut self, rule_engine: &GameRuleEngine) {
        for item in &mut self.items {
            restore_item_nightly_uses(item, rule_engine);
        }
    }
}

/// 搜索结果类
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// 搜索到的目标类型（玩家、物品等）
    pub target_type: SearchResultType,
    /// 目标ID
    pub target_id: String,
    /// 目标名称
    pub target_name: String,
    /// 搜索结果是否可见
    pub is_visible: bool,
}

/// 搜索结果类型枚举
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SearchResultType {
    Player,
    Item,
}

/// 搜索目标类型（内部使用）
#[derive(Debug, Clone)]
pub enum SearchTarget {
    Player(String), // 玩家ID
    Item(String),   // 物品ID
}

/// 动作处理结果集合，包含多个ActionResult
#[derive(Debug, Clone)]
pub struct ActionResults {
    pub results: Vec<ActionResult>,
}

/// 动作处理结果，包含广播信息
#[derive(Debug, Clone)]
pub struct ActionResult {
    /// 动作处理结果数据
    pub data: JsonValue,
    /// 需要广播消息的玩家ID列表（包括发起者本人）
    pub broadcast_players: Vec<String>,
    /// 日志消息（必须提供）
    pub log_message: String,
    /// 消息类型
    pub message_type: MessageType,
    /// 动作处理时间戳
    pub timestamp: DateTime<Utc>,
    /// 是否向导演广播
    pub broadcast_to_director: bool,
    /// 是否向所有玩家广播，用于写入数据库
    pub broadcast_to_all: bool,
}

impl ActionResult {
    /// 创建新的动作处理结果
    fn new(
        data: JsonValue,
        broadcast_players: Vec<String>,
        log_message: String,
        log_type: MessageType,
        broadcast_to_director: bool,
    ) -> Self {
        Self {
            data,
            broadcast_players,
            log_message,
            message_type: log_type,
            timestamp: Utc::now(),
            broadcast_to_director,
            broadcast_to_all: false,
        }
    }

    /// 创建新的动作处理结果（带系统日志消息）
    pub fn new_system_message(
        data: JsonValue,
        broadcast_players: Vec<String>,
        log_message: String,
        broadcast_to_director: bool,
    ) -> Self {
        ActionResult::new(
            data,
            broadcast_players,
            log_message,
            MessageType::SystemNotice,
            broadcast_to_director,
        )
    }

    /// 创建新的动作处理结果（带用户定向日志消息）
    pub fn new_user_message(
        data: JsonValue,
        broadcast_players: Vec<String>,
        log_message: String,
        broadcast_to_director: bool,
    ) -> Self {
        ActionResult::new(
            data,
            broadcast_players,
            log_message,
            MessageType::UserDirected,
            broadcast_to_director,
        )
    }

    /// 创建新的动作处理结果（带Info类型提示消息）
    pub fn new_info_message(
        data: JsonValue,
        broadcast_players: Vec<String>,
        log_message: String,
        broadcast_to_director: bool,
    ) -> Self {
        ActionResult::new(
            data,
            broadcast_players,
            log_message,
            MessageType::Info,
            broadcast_to_director,
        )
    }

    /// 将单个ActionResult转换为ActionResults
    pub fn as_results(self) -> ActionResults {
        ActionResults {
            results: vec![self],
        }
    }
}

impl GameState {
    /// 创建新的游戏状态
    pub fn new(game_id: String, rules_config: JsonValue) -> Self {
        // 解析JSON规则为结构化的规则引擎
        let rules_json =
            serde_json::to_string(&rules_config).expect("Failed to serialize rules config");
        let rule_engine =
            GameRuleEngine::from_json(&rules_json).expect("Failed to parse game rules");

        Self {
            game_id,
            players: HashMap::new(),
            places: HashMap::new(),
            weather: 1.0,
            votes: HashMap::new(),
            rules_config,
            rule_engine,
            night_start_time: None,
            night_end_time: None,
            next_night_destroyed_places: Vec::new(),
            save_time: None,
            shop: Vec::new(),
            sell_prices: Vec::new(),
        }
    }

    /// 判断两个玩家是否为同队队友
    /// 规则：team_id 必须相等且 > 0；自己不算自己的队友；任一玩家不存在返回 false
    pub fn are_teammates(&self, a_id: &str, b_id: &str) -> bool {
        if a_id == b_id {
            return false;
        }
        let (a, b) = match (self.players.get(a_id), self.players.get(b_id)) {
            (Some(a), Some(b)) => (a, b),
            _ => return false,
        };
        match (a.team_id, b.team_id) {
            (Some(ta), Some(tb)) => ta > 0 && ta == tb,
            _ => false,
        }
    }
}

// 为GameState实现自定义反序列化
impl<'de> Deserialize<'de> for GameState {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // 定义一个临时结构体用于反序列化
        #[derive(Deserialize)]
        struct GameStateHelper {
            game_id: String,
            players: HashMap<String, Player>,
            places: HashMap<String, Place>,
            weather: f64,
            votes: HashMap<String, String>,
            rules_config: JsonValue,
            night_start_time: Option<DateTime<Utc>>,
            night_end_time: Option<DateTime<Utc>>,
            next_night_destroyed_places: Vec<String>,
            save_time: Option<DateTime<Utc>>,
            #[serde(default)]
            shop: Vec<ShopListing>,
            #[serde(default)]
            sell_prices: Vec<SellPriceEntry>,
        }

        let helper = GameStateHelper::deserialize(deserializer)?;

        // 从 rules_config 重新创建 rule_engine
        let rules_json =
            serde_json::to_string(&helper.rules_config).map_err(serde::de::Error::custom)?;
        let rule_engine =
            GameRuleEngine::from_json(&rules_json).map_err(serde::de::Error::custom)?;

        // 旧存档玩家没有 max_backpack_items 字段（serde default 为 0），回填为规则初始值
        let mut players = helper.players;
        let default_backpack = rule_engine.player_config.max_backpack_items;
        for player in players.values_mut() {
            if player.max_backpack_items == 0 {
                player.max_backpack_items = default_backpack;
            }
        }

        Ok(GameState {
            game_id: helper.game_id,
            players,
            places: helper.places,
            weather: helper.weather,
            votes: helper.votes,
            rules_config: helper.rules_config,
            rule_engine,
            night_start_time: helper.night_start_time,
            night_end_time: helper.night_end_time,
            next_night_destroyed_places: helper.next_night_destroyed_places,
            save_time: helper.save_time,
            shop: helper.shop,
            sell_prices: helper.sell_prices,
        })
    }
}

#[cfg(test)]
mod are_teammates_tests {
    use super::*;
    use crate::game::game_rule_engine::GameRuleEngine;

    fn build_state_with_players(pairs: &[(&str, u32)]) -> GameState {
        let rules_json = r#"{
            "map": {"places": ["loc"], "safe_places": []},
            "player": {"max_life": 100, "max_strength": 100, "daily_life_recovery": 0, "daily_strength_recovery": 40, "search_cooldown": 30, "max_backpack_items": 6, "unarmed_damage": 5},
            "action_costs": {"move": 5, "search": 5, "pick": 0, "attack": 0, "equip": 0, "use": 0, "throw": 0, "deliver": 10},
            "rest_mode": {"life_recovery": 25, "strength_recovery": 1000, "max_moves": 1},
            "death_item_disposition": "killer_takes_loot",
            "teammate_behavior": 0,
            "items_config": {"rarity_levels": [], "items": {}, "upgrade_recipes": {}}
        }"#;
        let rules_value: JsonValue =
            serde_json::from_str(rules_json).expect("rules JSON must parse");
        let engine = GameRuleEngine::from_json(rules_json).unwrap();
        let mut state = GameState::new("game1".to_string(), rules_value);
        state.rule_engine = engine;
        for (id, team) in pairs {
            let mut p = Player::new(
                id.to_string(),
                format!("name_{}", id),
                "pw".to_string(),
                *team,
                &state.rule_engine,
            );
            p.location = "loc".to_string();
            state.players.insert(id.to_string(), p);
        }
        state
    }

    #[test]
    fn same_positive_team_id_are_teammates() {
        let s = build_state_with_players(&[("a", 1), ("b", 1)]);
        assert!(s.are_teammates("a", "b"));
        assert!(s.are_teammates("b", "a")); // symmetric
    }

    #[test]
    fn different_team_ids_not_teammates() {
        let s = build_state_with_players(&[("a", 1), ("b", 2)]);
        assert!(!s.are_teammates("a", "b"));
    }

    #[test]
    fn zero_team_id_never_teammate_even_if_equal() {
        let s = build_state_with_players(&[("a", 0), ("b", 0)]);
        assert!(!s.are_teammates("a", "b"));
    }

    #[test]
    fn self_is_not_own_teammate() {
        let s = build_state_with_players(&[("a", 1)]);
        assert!(!s.are_teammates("a", "a"));
    }

    #[test]
    fn unknown_player_id_not_teammate() {
        let s = build_state_with_players(&[("a", 1)]);
        assert!(!s.are_teammates("a", "ghost"));
        assert!(!s.are_teammates("ghost", "a"));
    }
}
