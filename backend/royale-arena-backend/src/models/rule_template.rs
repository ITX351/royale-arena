use serde::{Deserialize, Serialize};
use crate::models::rules::GameRules;

#[derive(Serialize, Deserialize, Clone)]
pub struct RuleTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub rules: GameRules,
}

impl RuleTemplate {
    pub fn new(id: String, name: String, description: String, rules: GameRules) -> Self {
        Self {
            id,
            name,
            description,
            rules,
        }
    }
    
    /// 创建一个默认的规则模版
    pub fn default_template() -> Self {
        Self::new(
            "default".to_string(),
            "默认规则".to_string(),
            "标准的大逃杀游戏规则".to_string(),
            GameRules::default(),
        )
    }
    
    /// 创建一个快速游戏的规则模版
    pub fn quick_game_template() -> Self {
        let mut rules = GameRules::default();
        rules.game_duration = 10; // 10分钟快速游戏
        rules.search_interval = 15; // 更短的搜索间隔
        
        Self::new(
            "quick".to_string(),
            "快速游戏".to_string(),
            "适合快速体验的游戏规则".to_string(),
            rules,
        )
    }
    
    /// 创建一个持久战的规则模版
    pub fn endurance_template() -> Self {
        let mut rules = GameRules::default();
        rules.max_life = 150; // 更高的生命值
        rules.max_strength = 150; // 更高的体力值
        rules.game_duration = 30; // 更长的游戏时间
        
        Self::new(
            "endurance".to_string(),
            "持久战".to_string(),
            "更长的游戏时间和更高的属性值".to_string(),
            rules,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rule_template_creation() {
        let template = RuleTemplate::default_template();
        
        assert_eq!(template.id, "default");
        assert_eq!(template.name, "默认规则");
        assert_eq!(template.description, "标准的大逃杀游戏规则");
    }

    #[test]
    fn test_quick_game_template() {
        let template = RuleTemplate::quick_game_template();
        
        assert_eq!(template.id, "quick");
        assert_eq!(template.name, "快速游戏");
        assert_eq!(template.rules.game_duration, 10);
        assert_eq!(template.rules.search_interval, 15);
    }

    #[test]
    fn test_endurance_template() {
        let template = RuleTemplate::endurance_template();
        
        assert_eq!(template.id, "endurance");
        assert_eq!(template.name, "持久战");
        assert_eq!(template.rules.max_life, 150);
        assert_eq!(template.rules.max_strength, 150);
        assert_eq!(template.rules.game_duration, 30);
    }

    #[test]
    fn test_rule_template_serialization() {
        let template = RuleTemplate::default_template();
        let serialized = serde_json::to_string(&template).unwrap();
        let deserialized: RuleTemplate = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(template.id, deserialized.id);
        assert_eq!(template.name, deserialized.name);
        assert_eq!(template.description, deserialized.description);
        assert_eq!(template.rules.max_life, deserialized.rules.max_life);
        assert_eq!(template.rules.max_strength, deserialized.rules.max_strength);
    }
}