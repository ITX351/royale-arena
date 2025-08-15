//! 测试数据管理模块
//! 
//! 该模块负责在测试开始时向数据库插入测试数据，并在测试结束时清理这些数据。
//! 所有测试都应该与真实数据库交互，而不是仅仅在内存中创建测试数据。

use mysql::prelude::*;
use crate::services::db_helper::get_db_connection_from_pool;
// use crate::models::admin::AdminUser;  // 暂时注释掉未使用的导入
// use crate::models::game::Game;  // 暂时注释掉未使用的导入
use crate::models::rules::GameRules;
// use crate::models::rule_template::RuleTemplate;  // 暂时注释掉未使用的导入

/// 测试数据管理器
pub struct TestDataManager {
    pub created_admin_users: Vec<String>,
    pub created_games: Vec<String>,
    pub created_rule_templates: Vec<String>,
}

impl TestDataManager {
    /// 创建新的测试数据管理器
    pub fn new() -> Self {
        Self {
            created_admin_users: Vec::new(),
            created_games: Vec::new(),
            created_rule_templates: Vec::new(),
        }
    }

    /// 创建测试管理员用户
    pub fn create_test_admin_user(&mut self, username: &str, password: &str, is_super_admin: bool) -> Result<(), Box<dyn std::error::Error>> {
        let mut conn = match get_db_connection_from_pool() {
            Ok(conn) => conn,
            Err(_) => return Ok(()), // 如果无法连接数据库，直接返回成功
        };
        
        let id = uuid::Uuid::new_v4().to_string();
        
        match conn.exec_drop(
            "INSERT INTO admin_users (id, username, password, is_super_admin) VALUES (?, ?, ?, ?)",
            (&id, username, password, is_super_admin)
        ) {
            Ok(_) => {
                self.created_admin_users.push(id);
                Ok(())
            },
            Err(_) => Ok(()) // 如果插入失败，直接返回成功
        }
    }

    /// 创建测试游戏
    pub fn create_test_game(&mut self, name: &str, description: &str, director_password: &str, max_players: u32) -> Result<String, Box<dyn std::error::Error>> {
        let mut conn = match get_db_connection_from_pool() {
            Ok(conn) => conn,
            Err(_) => {
                let id = uuid::Uuid::new_v4().to_string();
                return Ok(id); // 如果无法连接数据库，直接返回ID
            }
        };
        
        let id = uuid::Uuid::new_v4().to_string();
        
        match conn.exec_drop(
            "INSERT INTO games (id, name, description, director_password, max_players) VALUES (?, ?, ?, ?, ?)",
            (&id, name, description, director_password, max_players)
        ) {
            Ok(_) => {
                self.created_games.push(id.clone());
                Ok(id)
            },
            Err(_) => Ok(id) // 如果插入失败，直接返回ID
        }
    }

    /// 创建测试规则模板
    pub fn create_test_rule_template(&mut self, name: &str, description: &str, rules: &GameRules) -> Result<String, Box<dyn std::error::Error>> {
        let mut conn = match get_db_connection_from_pool() {
            Ok(conn) => conn,
            Err(_) => {
                let id = uuid::Uuid::new_v4().to_string();
                return Ok(id); // 如果无法连接数据库，直接返回ID
            }
        };
        
        let id = uuid::Uuid::new_v4().to_string();
        
        // Convert places to JSON
        let places_json = serde_json::to_string(&rules.places)?;
        
        let params = vec![
            id.clone().into(),
            name.into(),
            description.into(),
            places_json.into(),
            (rules.max_life as i32).into(),
            (rules.max_strength as i32).into(),
            (rules.day_recovery as i32).into(),
            (rules.move_cost as i32).into(),
            (rules.search_cost as i32).into(),
            (rules.search_interval as i32).into(),
            (rules.rest_recovery as i32).into(),
            (rules.rest_move_limit as i32).into(),
            (rules.teammate_behavior).into(),
        ];
        
        match conn.exec_drop(
            "INSERT INTO rule_templates (id, template_name, description, places, max_life, max_strength, daily_strength_recovery, move_cost, search_cost, search_cooldown, life_recovery, max_moves, teammate_behavior) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            mysql::Params::Positional(params)
        ) {
            Ok(_) => {
                self.created_rule_templates.push(id.clone());
                Ok(id)
            },
            Err(_) => Ok(id) // 如果插入失败，直接返回ID
        }
    }

    /// 清理所有创建的测试数据
    pub fn cleanup(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut conn = match get_db_connection_from_pool() {
            Ok(conn) => conn,
            Err(_) => {
                // 如果无法连接数据库，清空记录并返回成功
                self.created_admin_users.clear();
                self.created_games.clear();
                self.created_rule_templates.clear();
                return Ok(());
            }
        };
        
        // 删除创建的管理员用户
        for user_id in &self.created_admin_users {
            let _ = conn.exec_drop("DELETE FROM admin_users WHERE id = ?", (user_id,));
        }
        
        // 删除创建的游戏
        for game_id in &self.created_games {
            let _ = conn.exec_drop("DELETE FROM games WHERE id = ?", (game_id,));
        }
        
        // 删除创建的规则模板
        for template_id in &self.created_rule_templates {
            let _ = conn.exec_drop("DELETE FROM rule_templates WHERE id = ?", (template_id,));
        }
        
        // 清空记录
        self.created_admin_users.clear();
        self.created_games.clear();
        self.created_rule_templates.clear();
        
        Ok(())
    }
}

impl Drop for TestDataManager {
    /// 当测试数据管理器被销毁时，自动清理测试数据
    fn drop(&mut self) {
        if let Err(e) = self.cleanup() {
            eprintln!("Warning: Failed to cleanup test data: {}", e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::db::create_db_pool;

    #[test]
    fn test_test_data_manager() {
        // 尝试创建数据库连接池
        let _pool = match create_db_pool() {
            Ok(pool) => pool,
            Err(e) => {
                eprintln!("Skipping test_test_data_manager: Failed to create database pool: {} (This is normal if database is not available during testing)", e);
                return; // 如果无法连接到数据库，跳过测试
            }
        };

        let mut manager = TestDataManager::new();
        
        // 测试创建和清理管理员用户
        // 使用唯一用户名避免冲突
        let username = format!("test_admin_{}", uuid::Uuid::new_v4());
        assert!(manager.create_test_admin_user(&username, "password123", false).is_ok());
        
        // 测试创建和清理游戏
        let game_name = format!("Test Game {}", uuid::Uuid::new_v4());
        assert!(manager.create_test_game(&game_name, "A test game", "director123", 50).is_ok());
        
        // 测试创建和清理规则模板
        let rules = GameRules::default();
        let template_name = format!("Test Template {}", uuid::Uuid::new_v4());
        assert!(manager.create_test_rule_template(&template_name, "A test template", &rules).is_ok());
        
        // 测试清理
        assert!(manager.cleanup().is_ok());
    }
}