//! WebSocket相关模型定义

use crate::game::game_rule_engine::{GameRuleEngine, Item};
use crate::game::models::MessageType;
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
    pub rules_config: serde_json::Value,
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

        Self {
            id,
            name,
            password,
            location: String::new(),
            life: max_life,
            strength: max_strength,
            max_life,
            max_strength,
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
    pub fn set_bleed_effect(&mut self, damage: i32) {
        self.bleed_damage = damage;
    }

    /// 清除持续伤害效果
    pub fn clear_bleed_effect(&mut self) {
        self.bleed_damage = 0;
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
    pub fn daily_reset(&mut self) {
        self.rest_mode = true;
        self.rest_moves_used = 0;
        self.last_search_result = None;
        self.is_bound = false;
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
    /// 是否可见（受天气影响）
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
    pub data: serde_json::Value,
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
        data: serde_json::Value,
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
        data: serde_json::Value,
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
        data: serde_json::Value,
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
        data: serde_json::Value,
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

    /// 创建用于返回给前端的数据结构，排除`broadcast_players`字段
    pub fn to_client_response(&self) -> serde_json::Value {
        serde_json::json!({
            "data": self.data,
            "log_message": self.log_message,
            "message_type": self.message_type,
            "timestamp": self.timestamp
        })
    }
}

impl GameState {
    /// 创建新的游戏状态
    pub fn new(game_id: String, rules_config: serde_json::Value) -> Self {
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
        }
    }

    /// 从已有游戏状态反序列化时重新创建规则引擎
    // pub fn rebuild_rule_engine(&mut self) {
    //     let rules_json =
    //         serde_json::to_string(&self.rules_config).expect("Failed to serialize rules config");
    //     self.rule_engine =
    //         GameRuleEngine::from_json(&rules_json).expect("Failed to parse game rules");
    // }

    /// 生成全局状态信息
    pub fn generate_global_state_info(&self) -> serde_json::Value {
        serde_json::json!({
            "weather": self.weather,
            "night_start_time": self.night_start_time,
            "night_end_time": self.night_end_time,
            "next_night_destroyed_places": self.next_night_destroyed_places,
            "rules_config": self.rules_config,
        })
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
            rules_config: serde_json::Value,
            night_start_time: Option<DateTime<Utc>>,
            night_end_time: Option<DateTime<Utc>>,
            next_night_destroyed_places: Vec<String>,
            save_time: Option<DateTime<Utc>>,
        }

        let helper = GameStateHelper::deserialize(deserializer)?;

        // 从 rules_config 重新创建 rule_engine
        let rules_json =
            serde_json::to_string(&helper.rules_config).map_err(serde::de::Error::custom)?;
        let rule_engine =
            GameRuleEngine::from_json(&rules_json).map_err(serde::de::Error::custom)?;

        Ok(GameState {
            game_id: helper.game_id,
            players: helper.players,
            places: helper.places,
            weather: helper.weather,
            votes: helper.votes,
            rules_config: helper.rules_config,
            rule_engine,
            night_start_time: helper.night_start_time,
            night_end_time: helper.night_end_time,
            next_night_destroyed_places: helper.next_night_destroyed_places,
            save_time: helper.save_time,
        })
    }
}
