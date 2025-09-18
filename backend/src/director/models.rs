use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use crate::game::models::GameStatus;

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