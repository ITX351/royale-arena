use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::str::FromStr;

/// 游戏状态枚举
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "status", rename_all = "lowercase")]
pub enum GameStatus {
    #[serde(rename = "waiting")]
    Waiting,
    #[serde(rename = "running")]
    Running,
    #[serde(rename = "paused")]
    Paused,
    #[serde(rename = "ended")]
    Ended,
    #[serde(rename = "hidden")]
    Hidden,
    #[serde(rename = "deleted")]
    Deleted,
}

impl FromStr for GameStatus {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "waiting" => Ok(GameStatus::Waiting),
            "running" => Ok(GameStatus::Running),
            "paused" => Ok(GameStatus::Paused),
            "ended" => Ok(GameStatus::Ended),
            "hidden" => Ok(GameStatus::Hidden),
            "deleted" => Ok(GameStatus::Deleted),
            _ => Err(()),
        }
    }
}

/// 游戏实体模型
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct Game {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub director_password: String,
    pub max_players: i32,
    pub status: GameStatus,
    pub rules_config: serde_json::Value, // 修改：替换 rule_template_id 为 rules_config，且为非Option类型
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 游戏列表项（用于列表查询）
#[derive(Debug, Serialize)]
pub struct GameListItem {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub status: GameStatus,
    pub player_count: i32,
    pub max_players: i32,
    pub created_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub director_password: Option<String>,
}

/// 游戏查询结果（用于query_as宏）
#[derive(Debug, FromRow)]
pub struct GameQueryResult {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub status: String,  // MySQL ENUM在SQLx中作为字符串处理更可靠
    pub max_players: i32,
    pub created_at: DateTime<Utc>,
    pub player_count: i64,
    pub director_password: Option<String>,
}

impl From<GameQueryResult> for GameListItem {
    fn from(result: GameQueryResult) -> Self {
        let status = result.status.parse().unwrap_or(GameStatus::Waiting);
        
        Self {
            id: result.id,
            name: result.name,
            description: result.description,
            status,
            player_count: result.player_count as i32,
            max_players: result.max_players,
            created_at: result.created_at,
            director_password: result.director_password,
        }
    }
}

/// 带规则信息的游戏详情
#[derive(Debug, Serialize)]
pub struct GameWithRules {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub status: GameStatus,
    pub player_count: i32,
    pub max_players: i32,
    pub created_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub director_password: Option<String>,
    // 修改：直接包含规则配置而非模板信息，且为非Option类型
    pub rules_config: serde_json::Value,
}

/// 创建游戏请求
#[derive(Debug, Deserialize)]
pub struct CreateGameRequest {
    pub name: String,
    pub description: Option<String>,
    pub director_password: String,
    pub max_players: i32,
    pub rule_template_id: String, // 修改：rule_template_id 现在是必需的
}

/// 更新游戏请求
#[derive(Debug, Deserialize)]
pub struct UpdateGameRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub director_password: Option<String>,
    pub max_players: Option<i32>,
    // 修改：移除 rule_template_id，添加 rules_config（内部使用）
    pub rules_config: Option<serde_json::Value>,
}

/// 游戏筛选类型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GameFilterType {
    /// 全部（不包括已隐藏和已删除）
    All,
    /// 活动中（等待中、进行中、已暂停）
    Active,
    /// 等待中
    Waiting,
    /// 进行中
    Running,
    /// 已结束
    Ended,
    /// 已隐藏
    Hidden,
    /// 已删除（管理员可见）
    Deleted,
}

impl GameFilterType {
    /// 获取筛选类型对应的游戏状态列表
    pub fn get_status_list(&self) -> Vec<GameStatus> {
        match self {
            GameFilterType::All => vec![
                GameStatus::Waiting,
                GameStatus::Running,
                GameStatus::Paused,
                GameStatus::Ended,
            ],
            GameFilterType::Active => vec![
                GameStatus::Waiting,
                GameStatus::Running,
                GameStatus::Paused,
            ],
            GameFilterType::Waiting => vec![GameStatus::Waiting],
            GameFilterType::Running => vec![GameStatus::Running],
            GameFilterType::Ended => vec![GameStatus::Ended],
            GameFilterType::Hidden => vec![GameStatus::Hidden],
            GameFilterType::Deleted => vec![GameStatus::Deleted],
        }
    }
}

/// 游戏列表查询参数
#[derive(Debug, Deserialize)]
pub struct GameListQuery {
    pub filter: Option<GameFilterType>,
}

impl CreateGameRequest {
    /// 验证创建游戏请求的参数
    pub fn validate(&self) -> Result<(), String> {
        if self.name.trim().is_empty() {
            return Err("游戏名称不能为空".to_string());
        }
        
        if self.name.len() > 100 {
            return Err("游戏名称不能超过100个字符".to_string());
        }
        
        if self.director_password.len() < 6 || self.director_password.len() > 50 {
            return Err("导演密码长度必须在6-50字符之间".to_string());
        }
        
        if self.max_players < 1 || self.max_players > 1000 {
            return Err("最大玩家数必须在1-1000之间".to_string());
        }
        
        if self.rule_template_id.trim().is_empty() {
            return Err("规则模板ID不能为空".to_string());
        }
        
        Ok(())
    }
}

impl UpdateGameRequest {
    /// 验证更新游戏请求的参数
    pub fn validate(&self) -> Result<(), String> {
        if let Some(ref name) = self.name {
            if name.trim().is_empty() {
                return Err("游戏名称不能为空".to_string());
            }
            if name.len() > 100 {
                return Err("游戏名称不能超过100个字符".to_string());
            }
        }
        
        if let Some(ref password) = self.director_password {
            if password.len() < 6 || password.len() > 50 {
                return Err("导演密码长度必须在6-50字符之间".to_string());
            }
        }
        
        if let Some(max_players) = self.max_players {
            if max_players < 1 || max_players > 1000 {
                return Err("最大玩家数必须在1-1000之间".to_string());
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_game_request_validation() {
        // 测试有效请求
        let valid_request = CreateGameRequest {
            name: "测试游戏".to_string(),
            description: Some("测试描述".to_string()),
            director_password: "password123".to_string(),
            max_players: 10,
            rule_template_id: "template-id".to_string(), // 修改：现在是必需的
        };
        assert!(valid_request.validate().is_ok());

        // 测试空名称
        let invalid_name = CreateGameRequest {
            name: "".to_string(),
            description: None,
            director_password: "password123".to_string(),
            max_players: 10,
            rule_template_id: "template-id".to_string(), // 修改：现在是必需的
        };
        assert!(invalid_name.validate().is_err());

        // 测试密码过短
        let invalid_password = CreateGameRequest {
            name: "测试游戏".to_string(),
            description: None,
            director_password: "123".to_string(),
            max_players: 10,
            rule_template_id: "template-id".to_string(), // 修改：现在是必需的
        };
        assert!(invalid_password.validate().is_err());

        // 测试玩家数超限
        let invalid_players = CreateGameRequest {
            name: "测试游戏".to_string(),
            description: None,
            director_password: "password123".to_string(),
            max_players: 1001,
            rule_template_id: "template-id".to_string(), // 修改：现在是必需的
        };
        assert!(invalid_players.validate().is_err());

        // 测试空模板ID
        let invalid_template_id = CreateGameRequest {
            name: "测试游戏".to_string(),
            description: None,
            director_password: "password123".to_string(),
            max_players: 10,
            rule_template_id: "".to_string(), // 修改：空模板ID
        };
        assert!(invalid_template_id.validate().is_err());
    }

    #[test]
    fn test_update_game_request_validation() {
        // 测试有效更新请求
        let valid_update = UpdateGameRequest {
            name: Some("新游戏名".to_string()),
            description: Some("新描述".to_string()),
            director_password: Some("newpassword".to_string()),
            max_players: Some(20),
            rules_config: Some(serde_json::json!({"test": "config"})),
        };
        assert!(valid_update.validate().is_ok());

        // 测试空更新请求
        let empty_update = UpdateGameRequest {
            name: None,
            description: None,
            director_password: None,
            max_players: None,
            rules_config: None,
        };
        assert!(empty_update.validate().is_ok());

        // 测试无效名称
        let invalid_name = UpdateGameRequest {
            name: Some("".to_string()),
            description: None,
            director_password: None,
            max_players: None,
            rules_config: None,
        };
        assert!(invalid_name.validate().is_err());
    }
}