use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Player {
    pub id: String,
    pub name: String,
    pub password: String, // 6-8位字母数字
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_player_creation() {
        let player = Player {
            id: "player1".to_string(),
            name: "Test Player".to_string(),
            password: "pass123".to_string(),
            team_id: 1,
            life: 100,
            strength: 100,
            location: "zone1".to_string(),
            things: vec!["item1".to_string(), "item2".to_string()],
            hands: vec!["weapon1".to_string()],
            able: true,
            injured: 0,
            vote: 1,
            ts: 1234567890,
            deliver: 0,
            rest: 0,
        };

        assert_eq!(player.id, "player1");
        assert_eq!(player.name, "Test Player");
        assert_eq!(player.password, "pass123");
        assert_eq!(player.life, 100);
        assert_eq!(player.strength, 100);
        assert_eq!(player.location, "zone1");
        assert_eq!(player.things, vec!["item1", "item2"]);
        assert_eq!(player.hands, vec!["weapon1"]);
        assert!(player.able);
        assert_eq!(player.injured, 0);
        assert_eq!(player.vote, 1);
        assert_eq!(player.ts, 1234567890);
        assert_eq!(player.deliver, 0);
        assert_eq!(player.rest, 0);
    }

    #[test]
    fn test_player_serialization() {
        let player = Player {
            id: "player1".to_string(),
            name: "Test Player".to_string(),
            password: "pass123".to_string(),
            team_id: 1,
            life: 80,
            strength: 90,
            location: "zone2".to_string(),
            things: vec!["item3".to_string(), "item4".to_string()],
            hands: vec!["weapon2".to_string()],
            able: false,
            injured: 1,
            vote: 2,
            ts: 1234567891,
            deliver: 1,
            rest: 1,
        };

        let serialized = serde_json::to_string(&player).unwrap();
        let deserialized: Player = serde_json::from_str(&serialized).unwrap();

        assert_eq!(player.id, deserialized.id);
        assert_eq!(player.name, deserialized.name);
        assert_eq!(player.password, deserialized.password);
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