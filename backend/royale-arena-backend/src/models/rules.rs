use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone)]
pub struct GameRules {
    // 游戏基本设置
    pub max_life: u32,        // 生命值上限，默认100
    pub max_strength: u32,    // 体力值上限，默认100
    pub day_recovery: u32,    // 白天恢复体力值，默认40
    pub rest_recovery: u32,   // 静养模式恢复生命值，默认25
    pub search_interval: u32, // 搜索行动间隔秒数，默认30
    pub rest_move_limit: u32, // 静养模式下移动行动限制，默认1
    pub game_duration: u32,   // 游戏时长分钟数，默认15-30分钟

    // 行动消耗设置
    pub move_cost: u32,   // 移动体力消耗，默认5
    pub search_cost: u32, // 搜索体力消耗，默认5

    // 地点列表
    pub places: Vec<String>, // 游戏地图地点列表

    // 其他设置
    pub enable_day_voting: bool, // 是否启用白天投票机制

    // 队友行为规则（位压缩存储）
    // 0-无限制，1-禁止队友伤害，2-禁止搜索到队友，4-允许观看队友状态，8-允许赠送队友物品
    pub teammate_behavior: i32, // 队友行为规则
}

impl GameRules {
    pub fn default() -> Self {
        Self {
            max_life: 100,
            max_strength: 100,
            day_recovery: 40,
            rest_recovery: 25,
            search_interval: 30,
            rest_move_limit: 1,
            game_duration: 15, // 默认15分钟
            move_cost: 5,
            search_cost: 5,
            places: vec![
                "码头".to_string(),
                "工厂".to_string(),
                "贫民窟".to_string(),
                "旅馆".to_string(),
                "教堂".to_string(),
                "市政厅".to_string(),
                "消防局".to_string(),
                "池塘".to_string(),
                "住宅区".to_string(),
                "灯塔".to_string(),
                "小巷".to_string(),
                "学校".to_string(),
                "隧道".to_string(),
                "山道".to_string(),
                "寺庙".to_string(),
                "靶场".to_string(),
                "医院".to_string(),
                "森林".to_string(),
                "海滩".to_string(),
                "墓园".to_string(),
                "井".to_string(),
                "研究中心".to_string(),
            ],
            enable_day_voting: true,
            teammate_behavior: 0, // 默认无限制
        }
    }

    /// 验证规则的有效性
    pub fn validate(&self) -> Result<(), String> {
        if self.max_life == 0 || self.max_life > 1000 {
            return Err("生命值上限必须在1-1000之间".to_string());
        }

        if self.max_strength == 0 || self.max_strength > 1000 {
            return Err("体力值上限必须在1-1000之间".to_string());
        }

        if self.day_recovery > self.max_strength {
            return Err("白天恢复体力值不能超过体力值上限".to_string());
        }

        if self.rest_recovery > self.max_life {
            return Err("静养模式恢复生命值不能超过生命值上限".to_string());
        }

        if self.search_interval == 0 || self.search_interval > 3600 {
            return Err("搜索行动间隔必须在1-3600秒之间".to_string());
        }

        if self.rest_move_limit == 0 || self.rest_move_limit > 100 {
            return Err("静养模式下移动行动限制必须在1-100之间".to_string());
        }

        if self.game_duration == 0 || self.game_duration > 1440 {
            return Err("游戏时长必须在1-1440分钟之间".to_string());
        }

        if self.move_cost == 0 || self.move_cost > self.max_strength {
            return Err("移动体力消耗必须在1-体力值上限之间".to_string());
        }

        if self.search_cost == 0 || self.search_cost > self.max_strength {
            return Err("搜索体力消耗必须在1-体力值上限之间".to_string());
        }

        if self.places.is_empty() {
            return Err("地图地点列表不能为空".to_string());
        }

        if self.places.len() > 100 {
            return Err("地图地点数量不能超过100个".to_string());
        }

        // 检查地点名称是否重复
        let mut place_set = std::collections::HashSet::new();
        for place in &self.places {
            if place.is_empty() {
                return Err("地点名称不能为空".to_string());
            }

            if place.len() > 50 {
                return Err("地点名称不能超过50个字符".to_string());
            }

            if !place_set.insert(place.clone()) {
                return Err(format!("地点名称重复: {}", place));
            }
        }

        // 验证队友行为规则值
        if self.teammate_behavior < 0 || self.teammate_behavior > 15 {
            return Err("队友行为规则值必须在0-15之间".to_string());
        }

        Ok(())
    }

    /// 检查是否禁止队友伤害
    pub fn is_teammate_harm_prohibited(&self) -> bool {
        self.teammate_behavior & 1 != 0
    }

    /// 检查是否禁止搜索到队友
    pub fn is_teammate_search_prohibited(&self) -> bool {
        self.teammate_behavior & 2 != 0
    }

    /// 检查是否允许观看队友状态
    pub fn is_teammate_view_allowed(&self) -> bool {
        self.teammate_behavior & 4 != 0
    }

    /// 检查是否允许赠送队友物品
    pub fn is_teammate_gift_allowed(&self) -> bool {
        self.teammate_behavior & 8 != 0
    }

    /// 创建自定义规则
    pub fn new(
        max_life: u32,
        max_strength: u32,
        day_recovery: u32,
        rest_recovery: u32,
        search_interval: u32,
        rest_move_limit: u32,
        game_duration: u32,
        move_cost: u32,
        search_cost: u32,
        places: Vec<String>,
        enable_day_voting: bool,
        teammate_behavior: i32,
    ) -> Result<Self, String> {
        let rules = Self {
            max_life,
            max_strength,
            day_recovery,
            rest_recovery,
            search_interval,
            rest_move_limit,
            game_duration,
            move_cost,
            search_cost,
            places,
            enable_day_voting,
            teammate_behavior,
        };

        // 验证规则
        rules.validate()?;

        Ok(rules)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_rules() {
        let rules = GameRules::default();

        assert_eq!(rules.max_life, 100);
        assert_eq!(rules.max_strength, 100);
        assert_eq!(rules.day_recovery, 40);
        assert_eq!(rules.rest_recovery, 25);
        assert_eq!(rules.search_interval, 30);
        assert_eq!(rules.rest_move_limit, 1);
        assert_eq!(rules.game_duration, 15);
        assert_eq!(rules.move_cost, 5);
        assert_eq!(rules.search_cost, 5);
        assert_eq!(rules.places.len(), 22);
        assert!(rules.enable_day_voting);
        assert_eq!(rules.teammate_behavior, 0);
    }

    #[test]
    fn test_rules_serialization() {
        let rules = GameRules::default();
        let serialized = serde_json::to_string(&rules).unwrap();
        let deserialized: GameRules = serde_json::from_str(&serialized).unwrap();

        assert_eq!(rules.max_life, deserialized.max_life);
        assert_eq!(rules.max_strength, deserialized.max_strength);
        assert_eq!(rules.day_recovery, deserialized.day_recovery);
        assert_eq!(rules.rest_recovery, deserialized.rest_recovery);
        assert_eq!(rules.search_interval, deserialized.search_interval);
        assert_eq!(rules.rest_move_limit, deserialized.rest_move_limit);
        assert_eq!(rules.game_duration, deserialized.game_duration);
        assert_eq!(rules.move_cost, deserialized.move_cost);
        assert_eq!(rules.search_cost, deserialized.search_cost);
        assert_eq!(rules.places, deserialized.places);
        assert_eq!(rules.enable_day_voting, deserialized.enable_day_voting);
        assert_eq!(rules.teammate_behavior, deserialized.teammate_behavior);
    }

    #[test]
    fn test_teammate_behavior_flags() {
        // 测试默认规则（无限制）
        let rules = GameRules::default();
        assert!(!rules.is_teammate_harm_prohibited());
        assert!(!rules.is_teammate_search_prohibited());
        assert!(!rules.is_teammate_view_allowed());
        assert!(!rules.is_teammate_gift_allowed());

        // 测试禁止队友伤害
        let mut rules = GameRules::default();
        rules.teammate_behavior = 1;
        assert!(rules.is_teammate_harm_prohibited());
        assert!(!rules.is_teammate_search_prohibited());
        assert!(!rules.is_teammate_view_allowed());
        assert!(!rules.is_teammate_gift_allowed());

        // 测试多个标志组合
        let mut rules = GameRules::default();
        rules.teammate_behavior = 1 | 2 | 8; // 禁止伤害 + 禁止搜索 + 允许赠送
        assert!(rules.is_teammate_harm_prohibited());
        assert!(rules.is_teammate_search_prohibited());
        assert!(!rules.is_teammate_view_allowed());
        assert!(rules.is_teammate_gift_allowed());
    }

    #[test]
    fn test_rules_validation() {
        // 测试有效的规则
        let rules = GameRules::default();
        assert!(rules.validate().is_ok());

        // 测试无效的生命值上限
        let mut rules = GameRules::default();
        rules.max_life = 0;
        assert!(rules.validate().is_err());

        let mut rules = GameRules::default();
        rules.max_life = 1001;
        assert!(rules.validate().is_err());

        // 测试无效的体力值上限
        let mut rules = GameRules::default();
        rules.max_strength = 0;
        assert!(rules.validate().is_err());

        let mut rules = GameRules::default();
        rules.max_strength = 1001;
        assert!(rules.validate().is_err());

        // 测试白天恢复体力值超过上限
        let mut rules = GameRules::default();
        rules.day_recovery = 150;
        assert!(rules.validate().is_err());

        // 测试静养模式恢复生命值超过上限
        let mut rules = GameRules::default();
        rules.rest_recovery = 150;
        assert!(rules.validate().is_err());

        // 测试无效的搜索行动间隔
        let mut rules = GameRules::default();
        rules.search_interval = 0;
        assert!(rules.validate().is_err());

        let mut rules = GameRules::default();
        rules.search_interval = 3601;
        assert!(rules.validate().is_err());

        // 测试无效的移动行动限制
        let mut rules = GameRules::default();
        rules.rest_move_limit = 0;
        assert!(rules.validate().is_err());

        let mut rules = GameRules::default();
        rules.rest_move_limit = 101;
        assert!(rules.validate().is_err());

        // 测试无效的游戏时长
        let mut rules = GameRules::default();
        rules.game_duration = 0;
        assert!(rules.validate().is_err());

        let mut rules = GameRules::default();
        rules.game_duration = 1441;
        assert!(rules.validate().is_err());

        // 测试无效的移动体力消耗
        let mut rules = GameRules::default();
        rules.move_cost = 0;
        assert!(rules.validate().is_err());

        let mut rules = GameRules::default();
        rules.move_cost = 150;
        assert!(rules.validate().is_err());

        // 测试无效的搜索体力消耗
        let mut rules = GameRules::default();
        rules.search_cost = 0;
        assert!(rules.validate().is_err());

        let mut rules = GameRules::default();
        rules.search_cost = 150;
        assert!(rules.validate().is_err());

        // 测试空地点列表
        let mut rules = GameRules::default();
        rules.places = vec![];
        assert!(rules.validate().is_err());

        // 测试过多的地点
        let mut rules = GameRules::default();
        rules.places = vec!["地点".to_string(); 101];
        assert!(rules.validate().is_err());

        // 测试空地点名称
        let mut rules = GameRules::default();
        rules.places = vec!["".to_string()];
        assert!(rules.validate().is_err());

        // 测试过长的地点名称
        let mut rules = GameRules::default();
        rules.places = vec!["A".repeat(51)];
        assert!(rules.validate().is_err());

        // 测试重复的地点名称
        let mut rules = GameRules::default();
        rules.places = vec!["地点1".to_string(), "地点1".to_string()];
        assert!(rules.validate().is_err());

        // 测试无效的队友行为规则值
        let mut rules = GameRules::default();
        rules.teammate_behavior = -1;
        assert!(rules.validate().is_err());

        let mut rules = GameRules::default();
        rules.teammate_behavior = 16;
        assert!(rules.validate().is_err());
    }

    #[test]
    fn test_custom_rules_creation() {
        // 测试创建有效的自定义规则
        let places = vec!["地点1".to_string(), "地点2".to_string()];
        let rules = GameRules::new(
            150, // max_life
            150, // max_strength
            50,  // day_recovery
            30,  // rest_recovery
            45,  // search_interval
            2,   // rest_move_limit
            30,  // game_duration
            10,  // move_cost
            10,  // search_cost
            places, true, // enable_day_voting
            15,   // teammate_behavior (所有限制)
        );

        assert!(rules.is_ok());
        let rules = rules.unwrap();
        assert_eq!(rules.max_life, 150);
        assert_eq!(rules.max_strength, 150);
        assert_eq!(rules.day_recovery, 50);
        assert_eq!(rules.rest_recovery, 30);
        assert_eq!(rules.search_interval, 45);
        assert_eq!(rules.rest_move_limit, 2);
        assert_eq!(rules.game_duration, 30);
        assert_eq!(rules.move_cost, 10);
        assert_eq!(rules.search_cost, 10);
        assert_eq!(rules.places.len(), 2);
        assert!(rules.enable_day_voting);
        assert_eq!(rules.teammate_behavior, 15);
        assert!(rules.is_teammate_harm_prohibited());
        assert!(rules.is_teammate_search_prohibited());
        assert!(rules.is_teammate_view_allowed());
        assert!(rules.is_teammate_gift_allowed());

        // 测试创建无效的自定义规则
        let places = vec!["地点1".to_string(), "地点2".to_string()];
        let rules = GameRules::new(
            0, // 无效的生命值上限
            150, 50, 30, 45, 2, 30, 10, 10, places, true, 0,
        );

        assert!(rules.is_err());
    }
}
