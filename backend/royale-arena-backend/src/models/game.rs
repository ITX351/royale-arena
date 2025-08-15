use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Game {
    pub id: String,
    pub name: String,
    pub description: String,
    pub status: String,                   // waiting|running|paused|ended
    pub rule_template_id: Option<String>, // 关联的规则模板ID
    pub phase: String,                    // day|night
    pub player_count: u32,
    pub max_players: u32,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub action_start_time: Option<String>,
    pub action_end_time: Option<String>,
    pub safe_zones: Vec<String>,
    pub weather: f64,
    pub announcements: Vec<String>,
}

impl Game {
    /// 创建新游戏
    pub fn new(id: String, name: String, description: String, max_players: u32) -> Self {
        Self {
            id,
            name,
            description,
            status: "waiting".to_string(),
            rule_template_id: None,
            phase: "day".to_string(),
            player_count: 0,
            max_players,
            start_time: None,
            end_time: None,
            action_start_time: None,
            action_end_time: None,
            safe_zones: vec![],
            weather: 0.0,
            announcements: vec![],
        }
    }

    /// 验证游戏字段的有效性
    pub fn validate(&self) -> Result<(), String> {
        if self.id.is_empty() {
            return Err("游戏ID不能为空".to_string());
        }

        if self.name.is_empty() {
            return Err("游戏名称不能为空".to_string());
        }

        if self.name.len() > 100 {
            return Err("游戏名称不能超过100个字符".to_string());
        }

        if self.description.len() > 1000 {
            return Err("游戏描述不能超过1000个字符".to_string());
        }

        if !["waiting", "running", "paused", "ended"].contains(&self.status.as_str()) {
            return Err("游戏状态必须是waiting, running, paused, ended之一".to_string());
        }

        if !["day", "night"].contains(&self.phase.as_str()) {
            return Err("游戏阶段必须是day或night".to_string());
        }

        if self.player_count > self.max_players {
            return Err("玩家数量不能超过最大玩家数".to_string());
        }

        if self.max_players == 0 || self.max_players > 1000 {
            return Err("最大玩家数必须在1-1000之间".to_string());
        }

        if self.weather < 0.0 || self.weather > 1.0 {
            return Err("天气值必须在0.0-1.0之间".to_string());
        }

        Ok(())
    }

    /// 检查游戏是否处于活跃状态（可加入）
    pub fn is_active(&self) -> bool {
        self.status == "waiting" || self.status == "running"
    }

    /// 更新游戏状态
    pub fn set_status(&mut self, status: &str) -> Result<(), String> {
        if !["waiting", "running", "paused", "ended"].contains(&status) {
            return Err("无效的游戏状态".to_string());
        }
        self.status = status.to_string();
        Ok(())
    }

    /// 更新游戏阶段
    pub fn set_phase(&mut self, phase: &str) -> Result<(), String> {
        if !["day", "night"].contains(&phase) {
            return Err("无效的游戏阶段".to_string());
        }
        self.phase = phase.to_string();
        Ok(())
    }

    /// 添加安全区域
    pub fn add_safe_zone(&mut self, zone: String) {
        if !self.safe_zones.contains(&zone) {
            self.safe_zones.push(zone);
        }
    }

    /// 移除安全区域
    pub fn remove_safe_zone(&mut self, zone: &str) {
        self.safe_zones.retain(|z| z != zone);
    }

    /// 添加公告
    pub fn add_announcement(&mut self, announcement: String) {
        self.announcements.push(announcement);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_creation() {
        let game = Game::new(
            "game1".to_string(),
            "Test Game".to_string(),
            "A test game".to_string(),
            100,
        );

        assert_eq!(game.id, "game1");
        assert_eq!(game.name, "Test Game");
        assert_eq!(game.description, "A test game");
        assert_eq!(game.status, "waiting");
        assert_eq!(game.phase, "day");
        assert_eq!(game.player_count, 0);
        assert_eq!(game.max_players, 100);
        assert!(game.rule_template_id.is_none());
        assert!(game.start_time.is_none());
        assert!(game.end_time.is_none());
        assert!(game.action_start_time.is_none());
        assert!(game.action_end_time.is_none());
        assert!(game.safe_zones.is_empty());
        assert_eq!(game.weather, 0.0);
        assert!(game.announcements.is_empty());
    }

    #[test]
    fn test_game_serialization() {
        let mut game = Game::new(
            "game1".to_string(),
            "Test Game".to_string(),
            "A test game".to_string(),
            100,
        );
        game.status = "running".to_string();
        game.rule_template_id = Some("template1".to_string());
        game.player_count = 50;
        game.start_time = Some("2023-01-01T00:00:00Z".to_string());
        game.action_start_time = Some("2023-01-01T01:00:00Z".to_string());
        game.action_end_time = Some("2023-01-01T02:00:00Z".to_string());
        game.safe_zones = vec!["zone1".to_string(), "zone2".to_string()];
        game.weather = 0.5;
        game.announcements = vec!["Welcome!".to_string()];

        let serialized = serde_json::to_string(&game).unwrap();
        let deserialized: Game = serde_json::from_str(&serialized).unwrap();

        assert_eq!(game.id, deserialized.id);
        assert_eq!(game.name, deserialized.name);
        assert_eq!(game.description, deserialized.description);
        assert_eq!(game.status, deserialized.status);
        assert_eq!(game.rule_template_id, deserialized.rule_template_id);
        assert_eq!(game.phase, deserialized.phase);
        assert_eq!(game.player_count, deserialized.player_count);
        assert_eq!(game.max_players, deserialized.max_players);
        assert_eq!(game.start_time, deserialized.start_time);
        assert_eq!(game.action_start_time, deserialized.action_start_time);
        assert_eq!(game.action_end_time, deserialized.action_end_time);
        assert_eq!(game.safe_zones, deserialized.safe_zones);
        assert_eq!(game.weather, deserialized.weather);
        assert_eq!(game.announcements, deserialized.announcements);
    }

    #[test]
    fn test_game_validation() {
        // 测试有效的游戏
        let game = Game::new(
            "game1".to_string(),
            "Test Game".to_string(),
            "A test game".to_string(),
            100,
        );
        assert!(game.validate().is_ok());

        // 测试无效的游戏ID
        let mut game = Game::new(
            "".to_string(),
            "Test Game".to_string(),
            "A test game".to_string(),
            100,
        );
        assert!(game.validate().is_err());

        // 测试无效的游戏名称
        let game = Game::new(
            "game1".to_string(),
            "".to_string(),
            "A test game".to_string(),
            100,
        );
        assert!(game.validate().is_err());

        // 测试过长的游戏名称
        let long_name = "A".repeat(101);
        let game = Game::new(
            "game1".to_string(),
            long_name,
            "A test game".to_string(),
            100,
        );
        assert!(game.validate().is_err());

        // 测试过长的游戏描述
        let long_description = "A".repeat(1001);
        let game = Game::new(
            "game1".to_string(),
            "Test Game".to_string(),
            long_description,
            100,
        );
        assert!(game.validate().is_err());

        // 测试无效的游戏状态
        let mut game = Game::new(
            "game1".to_string(),
            "Test Game".to_string(),
            "A test game".to_string(),
            100,
        );
        game.status = "invalid".to_string();
        assert!(game.validate().is_err());

        // 测试无效的游戏阶段
        let mut game = Game::new(
            "game1".to_string(),
            "Test Game".to_string(),
            "A test game".to_string(),
            100,
        );
        game.phase = "invalid".to_string();
        assert!(game.validate().is_err());

        // 测试玩家数量超过最大玩家数
        let mut game = Game::new(
            "game1".to_string(),
            "Test Game".to_string(),
            "A test game".to_string(),
            100,
        );
        game.player_count = 150;
        assert!(game.validate().is_err());

        // 测试无效的最大玩家数
        let game = Game::new(
            "game1".to_string(),
            "Test Game".to_string(),
            "A test game".to_string(),
            0,
        );
        assert!(game.validate().is_err());

        let game = Game::new(
            "game1".to_string(),
            "Test Game".to_string(),
            "A test game".to_string(),
            1001,
        );
        assert!(game.validate().is_err());

        // 测试无效的天气值
        let mut game = Game::new(
            "game1".to_string(),
            "Test Game".to_string(),
            "A test game".to_string(),
            100,
        );
        game.weather = -0.1;
        assert!(game.validate().is_err());

        let mut game = Game::new(
            "game1".to_string(),
            "Test Game".to_string(),
            "A test game".to_string(),
            100,
        );
        game.weather = 1.1;
        assert!(game.validate().is_err());
    }

    #[test]
    fn test_game_status_management() {
        let mut game = Game::new(
            "game1".to_string(),
            "Test Game".to_string(),
            "A test game".to_string(),
            100,
        );

        // 测试设置状态
        assert!(game.set_status("running").is_ok());
        assert_eq!(game.status, "running");

        assert!(game.set_status("invalid").is_err());
        assert_eq!(game.status, "running"); // 状态不应改变

        // 测试活跃状态检查
        game.set_status("waiting").unwrap();
        assert!(game.is_active());

        game.set_status("running").unwrap();
        assert!(game.is_active());

        game.set_status("paused").unwrap();
        assert!(!game.is_active());

        game.set_status("ended").unwrap();
        assert!(!game.is_active());
    }

    #[test]
    fn test_game_phase_management() {
        let mut game = Game::new(
            "game1".to_string(),
            "Test Game".to_string(),
            "A test game".to_string(),
            100,
        );

        // 测试设置阶段
        assert!(game.set_phase("night").is_ok());
        assert_eq!(game.phase, "night");

        assert!(game.set_phase("invalid").is_err());
        assert_eq!(game.phase, "night"); // 阶段不应改变
    }

    #[test]
    fn test_safe_zone_management() {
        let mut game = Game::new(
            "game1".to_string(),
            "Test Game".to_string(),
            "A test game".to_string(),
            100,
        );

        // 测试添加安全区域
        game.add_safe_zone("zone1".to_string());
        assert_eq!(game.safe_zones.len(), 1);
        assert_eq!(game.safe_zones[0], "zone1");

        // 测试添加重复的安全区域（应该不会重复添加）
        game.add_safe_zone("zone1".to_string());
        assert_eq!(game.safe_zones.len(), 1);

        // 测试添加不同的安全区域
        game.add_safe_zone("zone2".to_string());
        assert_eq!(game.safe_zones.len(), 2);

        // 测试移除安全区域
        game.remove_safe_zone("zone1");
        assert_eq!(game.safe_zones.len(), 1);
        assert_eq!(game.safe_zones[0], "zone2");
    }

    #[test]
    fn test_announcement_management() {
        let mut game = Game::new(
            "game1".to_string(),
            "Test Game".to_string(),
            "A test game".to_string(),
            100,
        );

        // 测试添加公告
        game.add_announcement("Welcome!".to_string());
        assert_eq!(game.announcements.len(), 1);
        assert_eq!(game.announcements[0], "Welcome!");

        // 测试添加更多公告
        game.add_announcement("Game started!".to_string());
        assert_eq!(game.announcements.len(), 2);
        assert_eq!(game.announcements[1], "Game started!");
    }
}
