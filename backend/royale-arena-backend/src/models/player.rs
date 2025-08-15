use serde::{Deserialize, Serialize};
use bcrypt::{hash, verify, DEFAULT_COST};

#[derive(Serialize, Deserialize, Clone)]
pub struct Player {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing)] // 密码在序列化时应被隐藏
    pub password_hash: String, // 存储密码哈希而不是明文密码
    pub team_id: u32,     // 队伍ID，用于标识玩家所属队伍，0表示无队伍
    pub life: u32,        // 生命值 (0-100)
    pub strength: u32,    // 体力值 (0-100)
    pub location: String, // 当前位置
    pub things: Vec<String>, // 拥有的道具列表
    pub hands: Vec<String>,  // 装备的道具列表
    pub able: bool,       // 是否可行动
    pub injured: u32,     // 是否受伤 (持续伤害标记)
    pub vote: u32,        // 持有的票数
    pub ts: u64,          // 上次搜索时间戳
    pub deliver: u32,     // 传音次数标记
    pub rest: u32,        // 静养模式标记
}

impl Player {
    /// 创建新玩家，自动对密码进行哈希处理
    pub fn new(id: String, name: String, password: String, team_id: u32) -> Result<Self, bcrypt::BcryptError> {
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
    
    /// 验证密码是否正确
    pub fn verify_password(&self, password: &str) -> Result<bool, bcrypt::BcryptError> {
        verify(password, &self.password_hash)
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
            1
        ).unwrap();

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
        let password = "test_password";
        let player = Player::new(
            "player1".to_string(),
            "Test Player".to_string(),
            password.to_string(),
            1
        ).unwrap();

        // 验证密码哈希不等于原始密码
        assert_ne!(player.password_hash, password);
        
        // 验证密码验证功能
        assert!(player.verify_password(password).unwrap());
        assert!(!player.verify_password("wrong_password").unwrap());
    }

    #[test]
    fn test_player_serialization() {
        let player = Player::new(
            "player1".to_string(),
            "Test Player".to_string(),
            "pass123".to_string(),
            1
        ).unwrap();

        let serialized = serde_json::to_string(&player).unwrap();
        // 密码哈希不应被序列化
        assert!(!serialized.contains("password_hash"));
        
        // 测试反序列化，需要提供password_hash字段
        let json_with_hash = format!("{{\"id\":\"player1\",\"name\":\"Test Player\",\"password_hash\":\"{}\",\"team_id\":1,\"life\":100,\"strength\":100,\"location\":\"起始位置\",\"things\":[],\"hands\":[],\"able\":true,\"injured\":0,\"vote\":1,\"ts\":0,\"deliver\":0,\"rest\":0}}", player.password_hash);
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
}