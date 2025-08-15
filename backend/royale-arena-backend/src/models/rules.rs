use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct GameRules {
    // 游戏基本设置
    pub max_life: u32,           // 生命值上限，默认100
    pub max_strength: u32,       // 体力值上限，默认100
    pub day_recovery: u32,       // 白天恢复体力值，默认40
    pub rest_recovery: u32,      // 静养模式恢复生命值，默认25
    pub search_interval: u32,    // 搜索行动间隔秒数，默认30
    pub rest_move_limit: u32,    // 静养模式下移动行动限制，默认1
    pub game_duration: u32,      // 游戏时长分钟数，默认15-30分钟
    
    // 行动消耗设置
    pub move_cost: u32,          // 移动体力消耗，默认5
    pub search_cost: u32,        // 搜索体力消耗，默认5
    
    // 地点列表
    pub places: Vec<String>,     // 游戏地图地点列表
    
    // 其他设置
    pub enable_day_voting: bool, // 是否启用白天投票机制
    
    // 队友行为规则（位压缩存储）
    // 0-无限制，1-禁止队友伤害，2-禁止搜索到队友，4-允许观看队友状态，8-允许赠送队友物品
    pub teammate_behavior: i32,  // 队友行为规则
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
                "码头".to_string(), "工厂".to_string(), "贫民窟".to_string(), "旅馆".to_string(),
                "教堂".to_string(), "市政厅".to_string(), "消防局".to_string(), "池塘".to_string(),
                "住宅区".to_string(), "灯塔".to_string(), "小巷".to_string(), "学校".to_string(),
                "隧道".to_string(), "山道".to_string(), "寺庙".to_string(), "靶场".to_string(),
                "医院".to_string(), "森林".to_string(), "海滩".to_string(), "墓园".to_string(),
                "井".to_string(), "研究中心".to_string()
            ],
            enable_day_voting: true,
            teammate_behavior: 0, // 默认无限制
        }
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
}