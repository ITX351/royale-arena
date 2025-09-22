//! WebSocket相关模型定义

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use crate::game::models::MessageType;

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

// /// WebSocket服务端消息
// #[derive(Debug, Clone, Deserialize, Serialize)]
// pub struct WebSocketServerMessage {
//     /// 消息类型
//     #[serde(rename = "type")]
//     pub message_type: WebSocketMessageType,
//     /// 消息数据
//     pub data: JsonValue,
// }

/// 游戏阶段枚举
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GamePhase {
    Day,
    Night,
}

/// 游戏状态类
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    /// 游戏ID
    pub game_id: String,
    /// 玩家状态映射，键为玩家ID
    pub players: HashMap<String, Player>,
    /// 地点状态映射，键为地点名称
    pub places: HashMap<String, Place>,
    /// 当前游戏阶段（白天/夜晚）
    pub game_phase: GamePhase,
    /// 天气条件（影响搜索可见性）
    pub weather: f64,
    /// 投票记录，键为投票者ID，值为被投票者ID
    pub votes: HashMap<String, String>,
    /// 游戏规则配置
    pub rules_config: serde_json::Value,
    /// 夜晚开始时间
    pub night_start_time: Option<DateTime<Utc>>,
    /// 夜晚结束时间
    pub night_end_time: Option<DateTime<Utc>>,
    /// 下一夜晚缩圈地点集合
    pub next_night_destroyed_places: Vec<String>,
}

/// 玩家类
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    /// 玩家ID
    pub id: String,
    /// 玩家名称
    pub name: String,
    /// 当前位置
    pub location: String,
    /// 当前生命值
    pub life: i32,
    /// 当前体力值
    pub strength: i32,
    /// 物品背包
    pub inventory: Vec<Item>,
    /// 当前装备的物品
    pub equipped_item: Option<String>,
    /// 当前手持的物品
    pub hand_item: Option<String>,
    /// 上一次搜索结果
    pub last_search_result: Option<SearchResult>,
    /// 是否存活
    pub is_alive: bool,
    /// 是否被捆绑（禁止行动）
    pub is_bound: bool,
    /// 是否处于静养模式
    pub rest_mode: bool,
    /// 上次搜索时间
    pub last_search_time: Option<DateTime<Utc>>,
    /// 当前持有票数
    pub votes: i32,
}

impl Player {
    /// 创建新的玩家
    pub fn new(id: String, name: String, _team_id: u32) -> Self {
        Self {
            id,
            name,
            location: String::new(),
            life: 100, // 默认生命值
            strength: 100, // 默认体力值
            inventory: Vec::new(),
            equipped_item: None,
            hand_item: None,
            last_search_result: None,
            is_alive: true,
            is_bound: false,
            rest_mode: false,
            last_search_time: None,
            votes: 0,
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
}

/// 物品类
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    /// 物品ID
    pub id: String,
    /// 物品名称
    pub name: String,
    /// 物品类型（武器、消耗品等）
    pub item_type: ItemType,
    /// 物品属性（伤害值、恢复值等）
    pub properties: serde_json::Value,
}

/// 物品类型枚举
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ItemType {
    Weapon,
    Consumable,
    Equipment,
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
}

impl ActionResult {
    /// 创建新的动作处理结果
    fn new(data: serde_json::Value, broadcast_players: Vec<String>, log_message: String, log_type: MessageType) -> Self {
        Self {
            data,
            broadcast_players,
            log_message,
            message_type: log_type,
            timestamp: Utc::now(),
        }
    }
    
    /// 创建新的动作处理结果（带系统日志消息）
    pub fn new_system_message(data: serde_json::Value, broadcast_players: Vec<String>, log_message: String) -> Self {
        ActionResult::new(data, broadcast_players, log_message, MessageType::SystemNotice)
    }
    
    /// 创建新的动作处理结果（带用户定向日志消息）
    pub fn new_user_message(data: serde_json::Value, broadcast_players: Vec<String>, log_message: String) -> Self {
        ActionResult::new(data, broadcast_players, log_message, MessageType::UserDirected)
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
        Self {
            game_id,
            players: HashMap::new(),
            places: HashMap::new(),
            game_phase: GamePhase::Day,
            weather: 1.0,
            votes: HashMap::new(),
            rules_config,
            night_start_time: None,
            night_end_time: None,
            next_night_destroyed_places: Vec::new(),
        }
    }

    /// 生成全局状态信息
    pub fn generate_global_state_info(&self) -> serde_json::Value {
        serde_json::json!({
            "game_phase": self.game_phase,
            "weather": self.weather,
            "night_start_time": self.night_start_time,
            "night_end_time": self.night_end_time,
            "next_night_destroyed_places": self.next_night_destroyed_places,
        })
    }
}