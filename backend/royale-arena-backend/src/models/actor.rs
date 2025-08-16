use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Actor {
    pub id: String,
    pub game_id: String,
    pub name: String,
    pub password: String,  // 按照规范明文存储
    pub team_id: u32,     // 队伍ID，用于标识玩家所属队伍，0表示无队伍
}

impl Actor {
    /// 创建新演员
    pub fn new(
        id: String,
        game_id: String,
        name: String,
        password: String,
        team_id: u32,
    ) -> Result<Self, String> {
        // 验证字段
        Self::validate_fields(&id, &game_id, &name, &password)?;

        Ok(Self {
            id,
            game_id,
            name,
            password,
            team_id,
        })
    }

    /// 验证演员字段的有效性
    pub fn validate_fields(id: &str, game_id: &str, name: &str, password: &str) -> Result<(), String> {
        if id.is_empty() {
            return Err("演员ID不能为空".to_string());
        }

        if game_id.is_empty() {
            return Err("游戏ID不能为空".to_string());
        }

        if name.is_empty() {
            return Err("演员名称不能为空".to_string());
        }

        if name.len() > 50 {
            return Err("演员名称不能超过50个字符".to_string());
        }

        if password.len() < 6 || password.len() > 8 {
            return Err("密码长度必须在6-8个字符之间".to_string());
        }

        // 检查密码是否只包含字母和数字
        if !password.chars().all(|c| c.is_ascii_alphanumeric()) {
            return Err("密码只能包含字母和数字".to_string());
        }

        Ok(())
    }

    /// 验证密码是否正确
    pub fn verify_password(&self, password: &str) -> bool {
        self.password == password
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_actor_creation() {
        let actor = Actor::new(
            "actor1".to_string(),
            "game1".to_string(),
            "Test Actor".to_string(),
            "pass123".to_string(),
            1,
        )
        .unwrap();

        assert_eq!(actor.id, "actor1");
        assert_eq!(actor.game_id, "game1");
        assert_eq!(actor.name, "Test Actor");
        assert_eq!(actor.password, "pass123");
        assert_eq!(actor.team_id, 1);
    }

    #[test]
    fn test_actor_password_verification() {
        let actor = Actor::new(
            "actor1".to_string(),
            "game1".to_string(),
            "Test Actor".to_string(),
            "pass123".to_string(),
            1,
        )
        .unwrap();

        // 验证密码验证功能
        assert!(actor.verify_password("pass123"));
        assert!(!actor.verify_password("wrongpass"));
    }

    #[test]
    fn test_actor_field_validation() {
        // 测试有效的演员创建
        assert!(Actor::new(
            "actor1".to_string(),
            "game1".to_string(),
            "Test Actor".to_string(),
            "pass123".to_string(),
            1
        )
        .is_ok());

        // 测试空ID
        assert!(Actor::new(
            "".to_string(),
            "game1".to_string(),
            "Test Actor".to_string(),
            "pass123".to_string(),
            1
        )
        .is_err());

        // 测试空游戏ID
        assert!(Actor::new(
            "actor1".to_string(),
            "".to_string(),
            "Test Actor".to_string(),
            "pass123".to_string(),
            1
        )
        .is_err());

        // 测试空名称
        assert!(Actor::new(
            "actor1".to_string(),
            "game1".to_string(),
            "".to_string(),
            "pass123".to_string(),
            1
        )
        .is_err());

        // 测试过长的名称
        let long_name = "A".repeat(51);
        assert!(Actor::new(
            "actor1".to_string(),
            "game1".to_string(),
            long_name,
            "pass123".to_string(),
            1
        )
        .is_err());

        // 测试过短的密码
        assert!(Actor::new(
            "actor1".to_string(),
            "game1".to_string(),
            "Test Actor".to_string(),
            "pass".to_string(),
            1
        )
        .is_err());

        // 测试过长的密码
        let long_password = "A".repeat(9);
        assert!(Actor::new(
            "actor1".to_string(),
            "game1".to_string(),
            "Test Actor".to_string(),
            long_password,
            1
        )
        .is_err());

        // 测试包含特殊字符的密码
        assert!(Actor::new(
            "actor1".to_string(),
            "game1".to_string(),
            "Test Actor".to_string(),
            "pass@123".to_string(),
            1
        )
        .is_err());
    }
}