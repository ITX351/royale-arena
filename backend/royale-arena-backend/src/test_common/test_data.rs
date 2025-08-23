//! 测试数据管理模块
//!
//! 该模块负责在测试开始时向数据库插入测试数据，并在测试结束时清理这些数据。
//! 所有测试都应该与真实数据库交互，而不是仅仅在内存中创建测试数据。

use crate::services::db::create_db_pool;
use mysql::prelude::*;
// use crate::models::admin::AdminUser;  // 暂时注释掉未使用的导入
// use crate::models::game::Game;  // 暂时注释掉未使用的导入
use crate::models::rules::GameRules;
// use crate::models::rule_template::RuleTemplate;  // 暂时注释掉未使用的导入
use std::sync::LazyLock;
use std::sync::Arc;

static DB_POOL: LazyLock<Arc<mysql::Pool>> = LazyLock::new(|| {
    let pool = create_db_pool().expect("Failed to create database pool for tests");
    Arc::new(pool)
});

/// 获取全局共享的数据库连接池
/// 在测试环境中，我们只创建一个连接池以避免端口冲突
fn get_shared_db_pool() -> Result<mysql::PooledConn, Box<dyn std::error::Error>> {
    Ok(DB_POOL.get_conn()?)
}

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
    pub fn create_test_admin_user(
        &mut self,
        username: &str,
        password: &str,
        is_super_admin: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut conn = get_shared_db_pool()?;

        let id = uuid::Uuid::new_v4().to_string();

        conn.exec_drop(
            "INSERT INTO admin_users (id, username, password, is_super_admin) VALUES (?, ?, ?, ?)",
            (&id, username, password, is_super_admin),
        )?;

        self.created_admin_users.push(id);
        Ok(())
    }

    /// 创建测试游戏
    pub fn create_test_game(
        &mut self,
        name: &str,
        description: &str,
        director_password: &str,
        max_players: u32,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let mut conn = get_shared_db_pool()?;

        let id = uuid::Uuid::new_v4().to_string();

        conn.exec_drop(
            "INSERT INTO games (id, name, description, director_password, max_players) VALUES (?, ?, ?, ?, ?)",
            (&id, name, description, director_password, max_players)
        )?;

        self.created_games.push(id.clone());
        Ok(id)
    }

    /// 创建测试规则模板
    pub fn create_test_rule_template(
        &mut self,
        name: &str,
        description: &str,
        rules: &GameRules,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let mut conn = get_shared_db_pool()?;

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

        conn.exec_drop(
            "INSERT INTO rule_templates (id, template_name, description, places, max_life, max_strength, daily_strength_recovery, move_cost, search_cost, search_cooldown, life_recovery, max_moves, teammate_behavior) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            mysql::Params::Positional(params)
        )?;

        self.created_rule_templates.push(id.clone());
        Ok(id)
    }

    /// 清理所有创建的测试数据
    pub fn cleanup(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut conn = get_shared_db_pool()?;

        // 删除创建的管理员用户
        for user_id in &self.created_admin_users {
            conn.exec_drop("DELETE FROM admin_users WHERE id = ?", (user_id,))?;
        }

        // 删除创建的游戏
        for game_id in &self.created_games {
            conn.exec_drop("DELETE FROM games WHERE id = ?", (game_id,))?;
        }

        // 删除创建的规则模板
        for template_id in &self.created_rule_templates {
            conn.exec_drop("DELETE FROM rule_templates WHERE id = ?", (template_id,))?;
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

    #[test]
    fn test_test_data_manager() {
        // 测试创建和清理管理员用户
        // 使用唯一用户名避免冲突
        let username = format!("test_admin_{}", uuid::Uuid::new_v4());
        
        let mut manager = TestDataManager::new();
        manager
            .create_test_admin_user(&username, "password123", false)
            .expect("Failed to create test admin user");

        // 测试创建和清理游戏
        let game_name = format!("Test Game {}", uuid::Uuid::new_v4());
        manager
            .create_test_game(&game_name, "A test game", "director123", 50)
            .expect("Failed to create test game");

        // 测试创建和清理规则模板
        let rules = GameRules::default();
        let template_name = format!("Test Template {}", uuid::Uuid::new_v4());
        manager
            .create_test_rule_template(&template_name, "A test template", &rules)
            .expect("Failed to create test rule template");

        // 测试清理
        manager.cleanup().expect("Failed to cleanup test data");
    }
}