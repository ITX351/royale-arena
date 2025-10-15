use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use crate::game::models::{GameStatus, SaveFileInfo};

/// 导演编辑游戏请求
#[derive(Debug, Deserialize)]
pub struct DirectorEditGameRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub max_players: Option<i32>,
    pub rules_config: Option<serde_json::Value>,
}

/// 批量添加演员请求
#[derive(Debug, Deserialize)]
pub struct BatchAddPlayersRequest {
    pub players: Vec<CreatePlayerRequest>,
}

/// 创建单个演员请求
#[derive(Debug, Deserialize)]
pub struct CreatePlayerRequest {
    pub player_name: String,
    pub password: String,
    pub team_id: Option<i32>, // 可选的队伍ID，默认为0
}

/// 批量删除演员请求
#[derive(Debug, Deserialize)]
pub struct BatchDeletePlayersRequest {
    pub player_ids: Vec<String>,
}

/// 演员信息
#[derive(Debug, Serialize, FromRow)]
pub struct PlayerInfo {
    pub id: String,
    pub name: String,
    pub password: String,
    pub game_id: String,
    pub team_id: i32,
}

/// 批量操作响应
#[derive(Debug, Serialize)]
pub struct BatchOperationResponse<T> {
    pub success: Vec<T>,
    pub failed: Vec<OperationFailure>,
}

/// 操作失败信息
#[derive(Debug, Serialize)]
pub struct OperationFailure {
    pub player_name: Option<String>,
    pub id: Option<String>,
    pub reason: String,
}

/// 演员列表响应
#[derive(Debug, Serialize)]
pub struct PlayersListResponse {
    pub players: Vec<PlayerInfo>,
}

/// 删除操作成功信息
#[derive(Debug, Serialize)]
pub struct DeleteSuccessInfo {
    pub id: String,
    pub name: String,
    pub message: String,
}

/// 导演更新游戏状态请求
#[derive(Debug, Deserialize)]
pub struct UpdateGameStatusRequest {
    /// 导演密码
    pub password: String,
    /// 目标游戏状态
    pub status: GameStatus,
    /// 存档文件名（可选，用于恢复游戏时指定从哪个存档恢复）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub save_file_name: Option<String>,
}

/// 手动存盘请求
#[derive(Debug, Deserialize)]
pub struct ManualSaveRequest {
    /// 导演密码
    pub password: String,
}

/// 手动存盘响应
#[derive(Debug, Serialize, Deserialize)]
pub struct ManualSaveResponse {
    /// 操作是否成功
    pub success: bool,
    /// 操作结果消息
    pub message: String,
    /// 存盘文件名
    pub save_file_name: String,
}

/// 存档列表查询响应
#[derive(Debug, Serialize, Deserialize)]
pub struct ListSaveFilesResponse {
    /// 操作是否成功
    pub success: bool,
    /// 存档文件信息列表
    pub data: Vec<SaveFileInfo>,
}

/// 导演更新游戏状态响应
#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateGameStatusResponse {
    /// 操作是否成功
    pub success: bool,
    /// 操作结果消息
    pub message: String,
    /// 存盘文件名（仅在暂停游戏时返回）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub save_file_name: Option<String>,
}

impl DirectorEditGameRequest {
    /// 验证导演编辑游戏请求的数据
    pub fn validate(&self) -> Result<(), String> {
        // 验证至少提供一个字段
        if self.name.is_none() && self.description.is_none() && self.max_players.is_none() && self.rules_config.is_none() {
            return Err("至少需要提供一个字段进行更新".to_string());
        }

        // 验证游戏名称
        if let Some(ref name) = self.name {
            if name.trim().is_empty() {
                return Err("游戏名称不能为空".to_string());
            }
            if name.len() > 100 {
                return Err("游戏名称不能超过100个字符".to_string());
            }
        }

        // 验证最大玩家数
        if let Some(max_players) = self.max_players {
            if max_players < 1 || max_players > 1000 {
                return Err("最大玩家数必须在1-1000之间".to_string());
            }
        }

        Ok(())
    }
}

impl CreatePlayerRequest {
    /// 验证创建演员请求的数据
    pub fn validate(&self) -> Result<(), String> {
        // 验证演员名称
        if self.player_name.trim().is_empty() {
            return Err("演员名称不能为空".to_string());
        }
        
        if self.player_name.len() > 50 {
            return Err("演员名称不能超过50个字符".to_string());
        }

        // 验证密码格式
        if self.password.len() < 6 || self.password.len() > 8 {
            return Err("密码长度必须为6-8位字符".to_string());
        }

        // 验证密码只包含字母和数字
        if !self.password.chars().all(|c| c.is_alphanumeric()) {
            return Err("密码只能包含字母和数字".to_string());
        }

        // 验证队伍ID
        if let Some(team_id) = self.team_id {
            if team_id < 0 {
                return Err("队伍ID不能为负数".to_string());
            }
        }

        Ok(())
    }

    /// 获取有效的队伍ID，如果为None则返回0
    pub fn get_team_id(&self) -> i32 {
        self.team_id.unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_director_edit_game_request_validation() {
        // 测试有效请求 - 更新名称
        let valid_request = DirectorEditGameRequest {
            name: Some("测试游戏".to_string()),
            description: None,
            max_players: None,
            rules_config: None,
        };
        assert!(valid_request.validate().is_ok());

        // 测试有效请求 - 更新多个字段
        let valid_request = DirectorEditGameRequest {
            name: Some("测试游戏".to_string()),
            description: Some("测试描述".to_string()),
            max_players: Some(50),
            rules_config: Some(serde_json::json!({"test": "value"})),
        };
        assert!(valid_request.validate().is_ok());

        // 测试空请求
        let empty_request = DirectorEditGameRequest {
            name: None,
            description: None,
            max_players: None,
            rules_config: None,
        };
        assert!(empty_request.validate().is_err());

        // 测试空名称
        let empty_name_request = DirectorEditGameRequest {
            name: Some("".to_string()),
            description: None,
            max_players: None,
            rules_config: None,
        };
        assert!(empty_name_request.validate().is_err());

        // 测试名称过长
        let long_name_request = DirectorEditGameRequest {
            name: Some("a".repeat(101)),
            description: None,
            max_players: None,
            rules_config: None,
        };
        assert!(long_name_request.validate().is_err());

        // 测试无效最大玩家数 - 过小
        let invalid_max_players_request = DirectorEditGameRequest {
            name: None,
            description: None,
            max_players: Some(0),
            rules_config: None,
        };
        assert!(invalid_max_players_request.validate().is_err());

        // 测试无效最大玩家数 - 过大
        let invalid_max_players_request = DirectorEditGameRequest {
            name: None,
            description: None,
            max_players: Some(1001),
            rules_config: None,
        };
        assert!(invalid_max_players_request.validate().is_err());
    }

    #[test]
    fn test_create_player_request_validation() {
        // 测试有效请求
        let valid_request = CreatePlayerRequest {
            player_name: "测试玩家".to_string(),
            password: "abc123".to_string(),
            team_id: Some(1),
        };
        assert!(valid_request.validate().is_ok());

        // 测试空名称
        let empty_name_request = CreatePlayerRequest {
            player_name: "".to_string(),
            password: "abc123".to_string(),
            team_id: None,
        };
        assert!(empty_name_request.validate().is_err());

        // 测试名称过长
        let long_name_request = CreatePlayerRequest {
            player_name: "a".repeat(51),
            password: "abc123".to_string(),
            team_id: None,
        };
        assert!(long_name_request.validate().is_err());

        // 测试密码过短
        let short_password_request = CreatePlayerRequest {
            player_name: "测试玩家".to_string(),
            password: "abc".to_string(),
            team_id: None,
        };
        assert!(short_password_request.validate().is_err());

        // 测试密码过长
        let long_password_request = CreatePlayerRequest {
            player_name: "测试玩家".to_string(),
            password: "abcdefghi".to_string(),
            team_id: None,
        };
        assert!(long_password_request.validate().is_err());

        // 测试密码包含特殊字符
        let special_char_password_request = CreatePlayerRequest {
            player_name: "测试玩家".to_string(),
            password: "abc@123".to_string(),
            team_id: None,
        };
        assert!(special_char_password_request.validate().is_err());

        // 测试负数队伍ID
        let negative_team_id_request = CreatePlayerRequest {
            player_name: "测试玩家".to_string(),
            password: "abc123".to_string(),
            team_id: Some(-1),
        };
        assert!(negative_team_id_request.validate().is_err());
    }

    #[test]
    fn test_get_team_id() {
        let request_with_team_id = CreatePlayerRequest {
            player_name: "测试玩家".to_string(),
            password: "abc123".to_string(),
            team_id: Some(5),
        };
        assert_eq!(request_with_team_id.get_team_id(), 5);

        let request_without_team_id = CreatePlayerRequest {
            player_name: "测试玩家".to_string(),
            password: "abc123".to_string(),
            team_id: None,
        };
        assert_eq!(request_without_team_id.get_team_id(), 0);
    }
}