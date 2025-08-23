use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct KillRecord {
    pub id: String,
    pub game_id: String,
    pub killer_id: Option<String>,  // 击杀者ID（可为空，表示非玩家击杀）
    pub victim_id: String,          // 被击杀者ID
    pub kill_time: DateTime<Utc>,   // 击杀时间
    pub cause: String,              // 击杀原因（如：武器、缩圈等）
    pub weapon: Option<String>,     // 使用的武器/方式
    pub location: Option<String>,   // 击杀地点
}

impl KillRecord {
    /// 创建新的击杀记录
    pub fn new(
        id: String,
        game_id: String,
        killer_id: Option<String>,
        victim_id: String,
        kill_time: DateTime<Utc>,
        cause: String,
        weapon: Option<String>,
        location: Option<String>,
    ) -> Self {
        Self {
            id,
            game_id,
            killer_id,
            victim_id,
            kill_time,
            cause,
            weapon,
            location,
        }
    }

    /// 验证击杀记录字段的有效性
    pub fn validate(&self) -> Result<(), String> {
        if self.id.is_empty() {
            return Err("击杀记录ID不能为空".to_string());
        }

        if self.game_id.is_empty() {
            return Err("游戏ID不能为空".to_string());
        }

        if self.victim_id.is_empty() {
            return Err("被击杀者ID不能为空".to_string());
        }

        if self.cause.is_empty() {
            return Err("击杀原因不能为空".to_string());
        }

        if self.cause.len() > 50 {
            return Err("击杀原因不能超过50个字符".to_string());
        }

        if let Some(weapon) = &self.weapon {
            if weapon.len() > 50 {
                return Err("武器名称不能超过50个字符".to_string());
            }
        }

        if let Some(location) = &self.location {
            if location.len() > 100 {
                return Err("击杀地点不能超过100个字符".to_string());
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_kill_record_creation() {
        let kill_time = Utc::now();
        let kill_record = KillRecord::new(
            "kill-1".to_string(),
            "game-1".to_string(),
            Some("killer-1".to_string()),
            "victim-1".to_string(),
            kill_time,
            "weapon".to_string(),
            Some("AK-47".to_string()),
            Some("工厂".to_string()),
        );

        assert_eq!(kill_record.id, "kill-1");
        assert_eq!(kill_record.game_id, "game-1");
        assert_eq!(kill_record.killer_id, Some("killer-1".to_string()));
        assert_eq!(kill_record.victim_id, "victim-1");
        assert_eq!(kill_record.kill_time, kill_time);
        assert_eq!(kill_record.cause, "weapon");
        assert_eq!(kill_record.weapon, Some("AK-47".to_string()));
        assert_eq!(kill_record.location, Some("工厂".to_string()));
    }

    #[test]
    fn test_kill_record_validation() {
        let kill_time = Utc::now();
        
        // 测试有效的击杀记录
        let kill_record = KillRecord::new(
            "kill-1".to_string(),
            "game-1".to_string(),
            Some("killer-1".to_string()),
            "victim-1".to_string(),
            kill_time,
            "weapon".to_string(),
            Some("AK-47".to_string()),
            Some("工厂".to_string()),
        );
        assert!(kill_record.validate().is_ok());

        // 测试空ID
        let kill_record = KillRecord::new(
            "".to_string(),
            "game-1".to_string(),
            Some("killer-1".to_string()),
            "victim-1".to_string(),
            kill_time,
            "weapon".to_string(),
            Some("AK-47".to_string()),
            Some("工厂".to_string()),
        );
        assert!(kill_record.validate().is_err());

        // 测试空游戏ID
        let kill_record = KillRecord::new(
            "kill-1".to_string(),
            "".to_string(),
            Some("killer-1".to_string()),
            "victim-1".to_string(),
            kill_time,
            "weapon".to_string(),
            Some("AK-47".to_string()),
            Some("工厂".to_string()),
        );
        assert!(kill_record.validate().is_err());

        // 测试空被击杀者ID
        let kill_record = KillRecord::new(
            "kill-1".to_string(),
            "game-1".to_string(),
            Some("killer-1".to_string()),
            "".to_string(),
            kill_time,
            "weapon".to_string(),
            Some("AK-47".to_string()),
            Some("工厂".to_string()),
        );
        assert!(kill_record.validate().is_err());

        // 测试空击杀原因
        let kill_record = KillRecord::new(
            "kill-1".to_string(),
            "game-1".to_string(),
            Some("killer-1".to_string()),
            "victim-1".to_string(),
            kill_time,
            "".to_string(),
            Some("AK-47".to_string()),
            Some("工厂".to_string()),
        );
        assert!(kill_record.validate().is_err());

        // 测试过长的击杀原因
        let long_cause = "A".repeat(51);
        let kill_record = KillRecord::new(
            "kill-1".to_string(),
            "game-1".to_string(),
            Some("killer-1".to_string()),
            "victim-1".to_string(),
            kill_time,
            long_cause,
            Some("AK-47".to_string()),
            Some("工厂".to_string()),
        );
        assert!(kill_record.validate().is_err());

        // 测试过长的武器名称
        let long_weapon = "A".repeat(51);
        let kill_record = KillRecord::new(
            "kill-1".to_string(),
            "game-1".to_string(),
            Some("killer-1".to_string()),
            "victim-1".to_string(),
            kill_time,
            "weapon".to_string(),
            Some(long_weapon),
            Some("工厂".to_string()),
        );
        assert!(kill_record.validate().is_err());

        // 测试过长的击杀地点
        let long_location = "A".repeat(101);
        let kill_record = KillRecord::new(
            "kill-1".to_string(),
            "game-1".to_string(),
            Some("killer-1".to_string()),
            "victim-1".to_string(),
            kill_time,
            "weapon".to_string(),
            Some("AK-47".to_string()),
            Some(long_location),
        );
        assert!(kill_record.validate().is_err());
    }
}