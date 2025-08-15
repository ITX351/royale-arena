use bcrypt::{DEFAULT_COST, hash, verify};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone)]
pub struct Player {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing)] // 密码在序列化时应被隐藏
    pub password_hash: String, // 存储密码哈希而不是明文密码
    pub team_id: u32,        // 队伍ID，用于标识玩家所属队伍，0表示无队伍
    pub life: u32,           // 生命值 (0-100)
    pub strength: u32,       // 体力值 (0-100)
    pub location: String,    // 当前位置
    pub things: Vec<String>, // 拥有的道具列表
    pub hands: Vec<String>,  // 装备的道具列表
    pub able: bool,          // 是否可行动
    pub injured: u32,        // 是否受伤 (持续伤害标记)
    pub vote: u32,           // 持有的票数
    pub ts: u64,             // 上次搜索时间戳
    pub deliver: u32,        // 传音次数标记
    pub rest: u32,           // 静养模式标记
}

impl Player {
    /// 创建新玩家，自动对密码进行哈希处理
    pub fn new(
        id: String,
        name: String,
        password: String,
        team_id: u32,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // 验证字段
        Self::validate_fields(&id, &name, &password)?;

        let password_hash = hash(password, DEFAULT_COST)?;
        Ok(Self {
            id,
            name,
            password_hash,
            team_id,
            life: 100,
            strength: 100,
            location: "起始位置".to_string(),
            things: Vec::new(),
            hands: Vec::new(),
            able: true,
            injured: 0,
            vote: 1,
            ts: 0,
            deliver: 0,
            rest: 0,
        })
    }

    /// 验证玩家字段的有效性
    pub fn validate_fields(id: &str, name: &str, password: &str) -> Result<(), String> {
        if id.is_empty() {
            return Err("玩家ID不能为空".to_string());
        }

        if name.is_empty() {
            return Err("玩家名称不能为空".to_string());
        }

        if name.len() > 50 {
            return Err("玩家名称不能超过50个字符".to_string());
        }

        if password.len() < 6 || password.len() > 20 {
            return Err("密码长度必须在6-20个字符之间".to_string());
        }

        // 检查密码是否只包含字母和数字
        if !password.chars().all(|c| c.is_ascii_alphanumeric()) {
            return Err("密码只能包含字母和数字".to_string());
        }

        Ok(())
    }

    /// 验证玩家状态的有效性
    pub fn validate(&self) -> Result<(), String> {
        if self.id.is_empty() {
            return Err("玩家ID不能为空".to_string());
        }

        if self.name.is_empty() {
            return Err("玩家名称不能为空".to_string());
        }

        if self.name.len() > 50 {
            return Err("玩家名称不能超过50个字符".to_string());
        }

        if self.life > 100 {
            return Err("生命值不能超过100".to_string());
        }

        if self.strength > 100 {
            return Err("体力值不能超过100".to_string());
        }

        if self.location.is_empty() {
            return Err("位置不能为空".to_string());
        }

        if self.location.len() > 100 {
            return Err("位置名称不能超过100个字符".to_string());
        }

        if self.injured > 100 {
            return Err("受伤值不能超过100".to_string());
        }

        if self.vote > 100 {
            return Err("票数不能超过100".to_string());
        }

        if self.deliver > 100 {
            return Err("传音次数不能超过100".to_string());
        }

        if self.rest > 100 {
            return Err("静养模式标记不能超过100".to_string());
        }

        Ok(())
    }

    /// 验证密码是否正确
    pub fn verify_password(&self, password: &str) -> Result<bool, bcrypt::BcryptError> {
        verify(password, &self.password_hash)
    }

    /// 更新玩家位置
    pub fn set_location(&mut self, location: String) -> Result<(), String> {
        if location.is_empty() {
            return Err("位置不能为空".to_string());
        }

        if location.len() > 100 {
            return Err("位置名称不能超过100个字符".to_string());
        }

        self.location = location;
        Ok(())
    }

    /// 更新玩家生命值
    pub fn set_life(&mut self, life: u32) -> Result<(), String> {
        if life > 100 {
            return Err("生命值不能超过100".to_string());
        }

        self.life = life;
        Ok(())
    }

    /// 更新玩家体力值
    pub fn set_strength(&mut self, strength: u32) -> Result<(), String> {
        if strength > 100 {
            return Err("体力值不能超过100".to_string());
        }

        self.strength = strength;
        Ok(())
    }

    /// 添加道具到背包
    pub fn add_thing(&mut self, thing: String) {
        self.things.push(thing);
    }

    /// 从背包移除道具
    pub fn remove_thing(&mut self, thing: &str) -> bool {
        if let Some(index) = self.things.iter().position(|x| x == thing) {
            self.things.remove(index);
            true
        } else {
            false
        }
    }

    /// 装备道具
    pub fn equip(&mut self, thing: String) -> Result<(), String> {
        if !self.things.contains(&thing) {
            return Err("道具不在背包中".to_string());
        }

        if self.hands.len() >= 2 {
            return Err("装备槽已满".to_string());
        }

        self.hands.push(thing);
        Ok(())
    }

    /// 卸下装备
    pub fn unequip(&mut self, thing: &str) -> bool {
        if let Some(index) = self.hands.iter().position(|x| x == thing) {
            self.hands.remove(index);
            true
        } else {
            false
        }
    }

    /// 检查是否可以行动
    pub fn can_act(&self) -> bool {
        self.able && self.life > 0 && self.strength > 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_player_creation() {
        let player = Player::new(
            "player1".to_string(),
            "Test Player".to_string(),
            "pass123".to_string(),
            1,
        )
        .unwrap();

        assert_eq!(player.id, "player1");
        assert_eq!(player.name, "Test Player");
        assert_eq!(player.life, 100);
        assert_eq!(player.strength, 100);
        assert_eq!(player.location, "起始位置");
        assert_eq!(player.team_id, 1);
        assert!(player.things.is_empty());
        assert!(player.hands.is_empty());
        assert!(player.able);
        assert_eq!(player.injured, 0);
        assert_eq!(player.vote, 1);
        assert_eq!(player.ts, 0);
        assert_eq!(player.deliver, 0);
        assert_eq!(player.rest, 0);
    }

    #[test]
    fn test_player_password_hashing() {
        let password = "testpassword"; // 使用不含特殊字符的密码
        let player = Player::new(
            "player1".to_string(),
            "Test Player".to_string(),
            password.to_string(),
            1,
        )
        .unwrap();

        // 验证密码哈希不等于原始密码
        assert_ne!(player.password_hash, password);

        // 验证密码验证功能
        assert!(player.verify_password(password).unwrap());
        assert!(!player.verify_password("wrongpassword").unwrap()); // 使用不含特殊字符的密码
    }

    #[test]
    fn test_player_serialization() {
        let player = Player::new(
            "player1".to_string(),
            "Test Player".to_string(),
            "pass123".to_string(),
            1,
        )
        .unwrap();

        let serialized = serde_json::to_string(&player).unwrap();
        // 密码哈希不应被序列化
        assert!(!serialized.contains("password_hash"));

        // 测试反序列化，需要提供password_hash字段
        let json_with_hash = format!(
            r#"{{"id":"player1","name":"Test Player","password_hash":"{}","team_id":1,"life":100,"strength":100,"location":"起始位置","things":[],"hands":[],"able":true,"injured":0,"vote":1,"ts":0,"deliver":0,"rest":0}}"#,
            player.password_hash
        );
        let deserialized: Player = serde_json::from_str(&json_with_hash).unwrap();

        assert_eq!(player.id, deserialized.id);
        assert_eq!(player.name, deserialized.name);
        assert_eq!(player.team_id, deserialized.team_id);
        assert_eq!(player.life, deserialized.life);
        assert_eq!(player.strength, deserialized.strength);
        assert_eq!(player.location, deserialized.location);
        assert_eq!(player.things, deserialized.things);
        assert_eq!(player.hands, deserialized.hands);
        assert_eq!(player.able, deserialized.able);
        assert_eq!(player.injured, deserialized.injured);
        assert_eq!(player.vote, deserialized.vote);
        assert_eq!(player.ts, deserialized.ts);
        assert_eq!(player.deliver, deserialized.deliver);
        assert_eq!(player.rest, deserialized.rest);
    }

    #[test]
    fn test_player_field_validation() {
        // 测试有效的玩家创建
        assert!(
            Player::new(
                "player1".to_string(),
                "Test Player".to_string(),
                "pass123".to_string(),
                1
            )
            .is_ok()
        );

        // 测试空ID
        assert!(
            Player::new(
                "".to_string(),
                "Test Player".to_string(),
                "pass123".to_string(),
                1
            )
            .is_err()
        );

        // 测试空名称
        assert!(
            Player::new(
                "player1".to_string(),
                "".to_string(),
                "pass123".to_string(),
                1
            )
            .is_err()
        );

        // 测试过长的名称
        let long_name = "A".repeat(51);
        assert!(Player::new("player1".to_string(), long_name, "pass123".to_string(), 1).is_err());

        // 测试过短的密码
        assert!(
            Player::new(
                "player1".to_string(),
                "Test Player".to_string(),
                "pass".to_string(),
                1
            )
            .is_err()
        );

        // 测试过长的密码
        let long_password = "A".repeat(21);
        assert!(
            Player::new(
                "player1".to_string(),
                "Test Player".to_string(),
                long_password,
                1
            )
            .is_err()
        );

        // 测试包含特殊字符的密码
        assert!(
            Player::new(
                "player1".to_string(),
                "Test Player".to_string(),
                "pass@123".to_string(),
                1
            )
            .is_err()
        );
    }

    #[test]
    fn test_player_state_validation() {
        let player = Player::new(
            "player1".to_string(),
            "Test Player".to_string(),
            "pass123".to_string(),
            1,
        )
        .unwrap();

        // 测试有效的玩家状态
        assert!(player.validate().is_ok());

        // 测试无效的生命值
        let mut player = player.clone();
        player.life = 150;
        assert!(player.validate().is_err());

        // 测试无效的体力值
        let mut player = player.clone();
        player.life = 100;
        player.strength = 150;
        assert!(player.validate().is_err());

        // 测试空位置
        let mut player = player.clone();
        player.strength = 100;
        player.location = "".to_string();
        assert!(player.validate().is_err());

        // 测试过长的位置名称
        let mut player = player.clone();
        player.location = "A".repeat(101);
        assert!(player.validate().is_err());
    }

    #[test]
    fn test_player_location_management() {
        let mut player = Player::new(
            "player1".to_string(),
            "Test Player".to_string(),
            "pass123".to_string(),
            1,
        )
        .unwrap();

        // 测试设置位置
        assert!(player.set_location("新位置".to_string()).is_ok());
        assert_eq!(player.location, "新位置");

        // 测试设置空位置
        assert!(player.set_location("".to_string()).is_err());
        assert_eq!(player.location, "新位置"); // 位置不应改变

        // 测试设置过长的位置
        assert!(player.set_location("A".repeat(101)).is_err());
        assert_eq!(player.location, "新位置"); // 位置不应改变
    }

    #[test]
    fn test_player_attribute_management() {
        let mut player = Player::new(
            "player1".to_string(),
            "Test Player".to_string(),
            "pass123".to_string(),
            1,
        )
        .unwrap();

        // 测试设置生命值
        assert!(player.set_life(80).is_ok());
        assert_eq!(player.life, 80);

        // 测试设置过高的生命值
        assert!(player.set_life(150).is_err());
        assert_eq!(player.life, 80); // 生命值不应改变

        // 测试设置体力值
        assert!(player.set_strength(90).is_ok());
        assert_eq!(player.strength, 90);

        // 测试设置过高的体力值
        assert!(player.set_strength(150).is_err());
        assert_eq!(player.strength, 90); // 体力值不应改变
    }

    #[test]
    fn test_player_inventory_management() {
        let mut player = Player::new(
            "player1".to_string(),
            "Test Player".to_string(),
            "pass123".to_string(),
            1,
        )
        .unwrap();

        // 测试添加道具
        player.add_thing("道具1".to_string());
        assert_eq!(player.things.len(), 1);
        assert_eq!(player.things[0], "道具1");

        // 测试添加更多道具
        player.add_thing("道具2".to_string());
        assert_eq!(player.things.len(), 2);
        assert_eq!(player.things[1], "道具2");

        // 测试移除存在的道具
        assert!(player.remove_thing("道具1"));
        assert_eq!(player.things.len(), 1);
        assert_eq!(player.things[0], "道具2");

        // 测试移除不存在的道具
        assert!(!player.remove_thing("道具3"));
        assert_eq!(player.things.len(), 1);
    }

    #[test]
    fn test_player_equipment_management() {
        let mut player = Player::new(
            "player1".to_string(),
            "Test Player".to_string(),
            "pass123".to_string(),
            1,
        )
        .unwrap();

        // 添加道具到背包
        player.add_thing("武器1".to_string());
        player.add_thing("武器2".to_string());

        // 测试装备存在的道具
        assert!(player.equip("武器1".to_string()).is_ok());
        assert_eq!(player.hands.len(), 1);
        assert_eq!(player.hands[0], "武器1");

        // 测试装备不存在的道具
        assert!(player.equip("武器3".to_string()).is_err());
        assert_eq!(player.hands.len(), 1);

        // 测试装备更多道具
        assert!(player.equip("武器2".to_string()).is_ok());
        assert_eq!(player.hands.len(), 2);
        assert_eq!(player.hands[1], "武器2");

        // 测试装备槽已满
        player.add_thing("武器3".to_string());
        assert!(player.equip("武器3".to_string()).is_err());
        assert_eq!(player.hands.len(), 2);

        // 测试卸下装备
        assert!(player.unequip("武器1"));
        assert_eq!(player.hands.len(), 1);
        assert_eq!(player.hands[0], "武器2");

        // 测试卸下不存在的装备
        assert!(!player.unequip("武器3"));
        assert_eq!(player.hands.len(), 1);
    }

    #[test]
    fn test_player_action_ability() {
        let mut player = Player::new(
            "player1".to_string(),
            "Test Player".to_string(),
            "pass123".to_string(),
            1,
        )
        .unwrap();

        // 测试正常状态下的行动能力
        assert!(player.can_act());

        // 测试不可行动状态
        player.able = false;
        assert!(!player.can_act());

        // 测试生命值为0
        player.able = true;
        player.life = 0;
        assert!(!player.can_act());

        // 测试体力值为0
        player.life = 100;
        player.strength = 0;
        assert!(!player.can_act());

        // 测试恢复正常
        player.strength = 100;
        assert!(player.can_act());
    }
}
