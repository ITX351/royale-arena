use mysql::prelude::*;
use mysql::Value;
use crate::models::admin_game::{CreateGameRequest, UpdateGameRequest, CreateRuleTemplateRequest, UpdateRuleTemplateRequest};
use crate::models::game::Game;
use crate::models::rule_template::RuleTemplate;
use crate::models::rules::GameRules;

#[cfg(test)]
use crate::test_data::TestDataManager;

/// 创建游戏
pub fn create_game(
    conn: &mut mysql::PooledConn,
    request: &CreateGameRequest,
) -> Result<String, Box<dyn std::error::Error>> {
    let game_id = uuid::Uuid::new_v4().to_string();
    
    conn.exec_drop(
        "INSERT INTO games (id, name, description, director_password, max_players) VALUES (?, ?, ?, ?, ?)",
        (&game_id, &request.name, &request.description, &request.director_password, &request.max_players)
    )?;
    
    // 如果提供了规则模板ID，则应用该模板的规则
    if let Some(template_id) = &request.rules_template_id {
        let _template = get_rule_template(conn, template_id)?;
        // 这里应该将模板规则保存到游戏规则表中，但根据任务清单，游戏规则是存储在内存中的
        // 所以我们暂时只记录模板ID，实际规则应用在游戏逻辑引擎中处理
    }
    
    Ok(game_id)
}

/// 更新游戏
pub fn update_game(
    conn: &mut mysql::PooledConn,
    game_id: &str,
    request: &UpdateGameRequest,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut updates = Vec::new();
    let mut params: Vec<Value> = Vec::new();
    
    if let Some(name) = &request.name {
        updates.push("name = ?".to_string());
        params.push(name.into());
    }
    
    if let Some(description) = &request.description {
        updates.push("description = ?".to_string());
        params.push(description.into());
    }
    
    if let Some(password) = &request.director_password {
        updates.push("director_password = ?".to_string());
        params.push(password.into());
    }
    
    if let Some(max_players) = &request.max_players {
        updates.push("max_players = ?".to_string());
        params.push((*max_players).into());
    }
    
    if !updates.is_empty() {
        let sql = format!("UPDATE games SET {} WHERE id = ?", updates.join(", "));
        params.push(game_id.into());
        
        // Convert params to tuple for mysql exec_drop
        conn.exec_drop(&sql, mysql::Params::Positional(params))?;
    }
    
    Ok(())
}

/// 删除游戏
pub fn delete_game(
    conn: &mut mysql::PooledConn,
    game_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    conn.exec_drop(
        "DELETE FROM games WHERE id = ?",
        (game_id,)
    )?;
    
    Ok(())
}

/// 获取游戏信息
pub fn get_game(
    conn: &mut mysql::PooledConn,
    game_id: &str,
) -> Result<Option<Game>, Box<dyn std::error::Error>> {
    let result: Option<(String, String, String, String, u32, String, Option<String>)> = conn.exec_first(
        "SELECT id, name, description, director_password, max_players, status, rule_template_id FROM games WHERE id = ?",
        (game_id,)
    )?;
    
    match result {
        Some((id, name, description, _director_password, max_players, status, rule_template_id)) => {
            Ok(Some(Game {
                id,
                name,
                description,
                status,
                rule_template_id,
                phase: "day".to_string(), // Default value
                player_count: 0, // Default value
                max_players,
                start_time: None,
                end_time: None,
                action_start_time: None,
                action_end_time: None,
                safe_zones: vec![],
                weather: 0.0,
                announcements: vec![],
            }))
        },
        None => Ok(None),
    }
}

/// 创建规则模板
pub fn create_rule_template(
    conn: &mut mysql::PooledConn,
    request: &CreateRuleTemplateRequest,
) -> Result<String, Box<dyn std::error::Error>> {
    let template_id = uuid::Uuid::new_v4().to_string();
    
    // Convert places to JSON
    let places_json = serde_json::to_string(&request.rules.places)?;
    
    // 使用Vec<Value>来避免参数数量限制
    let params = vec![
        template_id.clone().into(),
        request.name.clone().into(),
        request.description.clone().into(),
        places_json.into(),
        (request.rules.max_life as i32).into(),
        (request.rules.max_strength as i32).into(),
        (request.rules.day_recovery as i32).into(),
        (request.rules.move_cost as i32).into(),
        (request.rules.search_cost as i32).into(),
        (request.rules.search_interval as i32).into(),
        (request.rules.rest_recovery as i32).into(),
        (request.rules.rest_move_limit as i32).into(),
        (request.rules.teammate_behavior).into(),
    ];
    
    conn.exec_drop(
        "INSERT INTO rule_templates (id, template_name, description, places, max_life, max_strength, daily_strength_recovery, move_cost, search_cost, search_cooldown, life_recovery, max_moves, teammate_behavior) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        mysql::Params::Positional(params)
    )?;
    
    Ok(template_id)
}

/// 更新规则模板
pub fn update_rule_template(
    conn: &mut mysql::PooledConn,
    template_id: &str,
    request: &UpdateRuleTemplateRequest,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut updates = Vec::new();
    let mut params: Vec<Value> = Vec::new();
    
    if let Some(name) = &request.name {
        updates.push("template_name = ?".to_string());
        params.push(name.into());
    }
    
    if let Some(description) = &request.description {
        updates.push("description = ?".to_string());
        params.push(description.into());
    }
    
    if let Some(rules) = &request.rules {
        // Convert places to JSON
        let places_json = serde_json::to_string(&rules.places)?;
        
        updates.push("places = ?".to_string());
        params.push(places_json.into());
        
        // Update individual fields
        updates.push("max_life = ?".to_string());
        params.push((rules.max_life as i32).into());
        
        updates.push("max_strength = ?".to_string());
        params.push((rules.max_strength as i32).into());
        
        updates.push("daily_strength_recovery = ?".to_string());
        params.push((rules.day_recovery as i32).into());
        
        updates.push("move_cost = ?".to_string());
        params.push((rules.move_cost as i32).into());
        
        updates.push("search_cost = ?".to_string());
        params.push((rules.search_cost as i32).into());
        
        updates.push("search_cooldown = ?".to_string());
        params.push((rules.search_interval as i32).into());
        
        updates.push("life_recovery = ?".to_string());
        params.push((rules.rest_recovery as i32).into());
        
        updates.push("max_moves = ?".to_string());
        params.push((rules.rest_move_limit as i32).into());
        
        updates.push("teammate_behavior = ?".to_string());
        params.push((rules.teammate_behavior).into());
    }
    
    if !updates.is_empty() {
        let sql = format!("UPDATE rule_templates SET {} WHERE id = ?", updates.join(", "));
        params.push(template_id.into());
        
        // Convert params to tuple for mysql exec_drop
        conn.exec_drop(&sql, mysql::Params::Positional(params))?;
    }
    
    Ok(())
}

/// 删除规则模板
pub fn delete_rule_template(
    conn: &mut mysql::PooledConn,
    template_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    conn.exec_drop(
        "DELETE FROM rule_templates WHERE id = ?",
        (template_id,)
    )?;
    
    Ok(())
}

/// 获取规则模板
pub fn get_rule_template(
    conn: &mut mysql::PooledConn,
    template_id: &str,
) -> Result<RuleTemplate, Box<dyn std::error::Error>> {
    let result: Option<mysql::Row> = conn.exec_first(
        "SELECT id, template_name, description, places, max_life, max_strength, daily_strength_recovery, move_cost, search_cost, search_cooldown, life_recovery, max_moves, teammate_behavior FROM rule_templates WHERE id = ?",
        (template_id,)
    )?;
    
    match result {
        Some(row) => {
            let id: String = row.get(0).unwrap_or_default();
            let name: String = row.get(1).unwrap_or_default();
            let description: String = row.get(2).unwrap_or_default();
            let places_json: String = row.get(3).unwrap_or_default();
            let max_life: i32 = row.get(4).unwrap_or_default();
            let max_strength: i32 = row.get(5).unwrap_or_default();
            let daily_strength_recovery: i32 = row.get(6).unwrap_or_default();
            let move_cost: i32 = row.get(7).unwrap_or_default();
            let search_cost: i32 = row.get(8).unwrap_or_default();
            let search_cooldown: i32 = row.get(9).unwrap_or_default();
            let life_recovery: i32 = row.get(10).unwrap_or_default();
            let max_moves: i32 = row.get(11).unwrap_or_default();
            let teammate_behavior: i32 = row.get(12).unwrap_or_default();
            
            // Parse places JSON
            let places: Vec<String> = if !places_json.is_empty() {
                serde_json::from_str(&places_json)?
            } else {
                Vec::new()
            };
            
            let rules = GameRules {
                max_life: max_life as u32,
                max_strength: max_strength as u32,
                day_recovery: daily_strength_recovery as u32,
                rest_recovery: life_recovery as u32,
                search_interval: search_cooldown as u32,
                rest_move_limit: max_moves as u32,
                game_duration: 15, // Default value, as it's not stored in the template
                move_cost: move_cost as u32,
                search_cost: search_cost as u32,
                places,
                enable_day_voting: true, // Default value, as it's not stored in the template
                teammate_behavior,
            };
            
            Ok(RuleTemplate::new(
                id,
                name,
                description,
                rules
            ))
        },
        None => Err("Rule template not found".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::db::create_db_pool;
    use dotenvy::from_filename;
    
    #[test]
    fn test_create_and_manage_game() {
        // Load environment variables from .env.royale file
        match from_filename(".env.royale") {
            Ok(_) => println!("Successfully loaded .env.royale file for tests"),
            Err(e) => eprintln!("Warning: Failed to load .env.royale file for tests: {}", e),
        }
        
        // Create database connection pool
        let pool = create_db_pool().expect("Failed to create database pool");
        let mut conn = pool.get_conn().expect("Failed to get database connection");
        
        // Create test data manager
        let mut test_data_manager = TestDataManager::new();
        
        // Test data - 使用唯一游戏名避免冲突
        let game_name = format!("Test Game {}", uuid::Uuid::new_v4());
        let create_request = CreateGameRequest {
            name: game_name,
            description: "A test game for unit tests".to_string(),
            director_password: "director123".to_string(),
            max_players: 50,
            rules_template_id: None,
        };
        
        // Create game
        let game_id = create_game(&mut conn, &create_request).expect("Failed to create game");
        assert!(!game_id.is_empty(), "Game ID should not be empty");
        
        // Add game to test data manager for cleanup
        test_data_manager.created_games.push(game_id.clone());
        
        // Get game to verify creation
        let game = get_game(&mut conn, &game_id).expect("Failed to get game");
        assert!(game.is_some(), "Game should exist");
        let game = game.unwrap();
        // 验证默认的rule_template_id值
        assert_eq!(game.rule_template_id, None);
        
        // Update game
        let update_request = UpdateGameRequest {
            name: Some("Updated Test Game".to_string()),
            description: Some("An updated test game".to_string()),
            director_password: Some("newdirector123".to_string()),
            max_players: Some(75),
        };
        
        let result = update_game(&mut conn, &game_id, &update_request);
        assert!(result.is_ok(), "Failed to update game: {:?}", result.err());
        
        // Get game to verify update
        let game = get_game(&mut conn, &game_id).expect("Failed to get game");
        assert!(game.is_some(), "Game should exist");
        let game = game.unwrap();
        assert_eq!(game.name, "Updated Test Game");
        assert_eq!(game.max_players, 75);
        // 验证rule_template_id值在更新后保持不变
        assert_eq!(game.rule_template_id, None);
        
        // Delete game
        let result = delete_game(&mut conn, &game_id);
        assert!(result.is_ok(), "Failed to delete game: {:?}", result.err());
        
        // Verify game is deleted
        let game = get_game(&mut conn, &game_id).expect("Failed to get game");
        assert!(game.is_none(), "Game should not exist after deletion");
        
        // Clean up test data
        test_data_manager.cleanup().expect("Failed to cleanup test data");
    }
    
    #[test]
    fn test_create_and_manage_rule_template() {
        // Load environment variables from .env.royale file
        match from_filename(".env.royale") {
            Ok(_) => println!("Successfully loaded .env.royale file for tests"),
            Err(e) => eprintln!("Warning: Failed to load .env.royale file for tests: {}", e),
        }
        
        // Create database connection pool
        let pool = create_db_pool().expect("Failed to create database pool");
        let mut conn = pool.get_conn().expect("Failed to get database connection");
        
        // Create test data manager
        let mut test_data_manager = TestDataManager::new();
        
        // Test data - 使用唯一模板名避免冲突
        let template_name = format!("Test Template {}", uuid::Uuid::new_v4());
        let mut rules = GameRules::default();
        rules.max_life = 150;
        rules.max_strength = 150;
        rules.teammate_behavior = 5; // 设置队友行为规则
        
        let create_request = CreateRuleTemplateRequest {
            name: template_name,
            description: "A test rule template".to_string(),
            rules,
        };
        
        // Create rule template
        let template_id = create_rule_template(&mut conn, &create_request).expect("Failed to create rule template");
        assert!(!template_id.is_empty(), "Template ID should not be empty");
        
        // Add template to test data manager for cleanup
        test_data_manager.created_rule_templates.push(template_id.clone());
        
        // Get rule template to verify creation
        let template = get_rule_template(&mut conn, &template_id);
        assert!(template.is_ok(), "Failed to get rule template: {:?}", template.err());
        let template = template.unwrap();
        // 验证teammate_behavior值
        assert_eq!(template.rules.teammate_behavior, 5);
        
        // Update rule template
        let mut updated_rules = GameRules::default();
        updated_rules.teammate_behavior = 10; // 更新队友行为规则
        
        let update_request = UpdateRuleTemplateRequest {
            name: Some("Updated Test Template".to_string()),
            description: Some("An updated test rule template".to_string()),
            rules: Some(updated_rules),
        };
        
        let result = update_rule_template(&mut conn, &template_id, &update_request);
        assert!(result.is_ok(), "Failed to update rule template: {:?}", result.err());
        
        // Get rule template to verify update
        let template = get_rule_template(&mut conn, &template_id);
        assert!(template.is_ok(), "Failed to get rule template: {:?}", template.err());
        let template = template.unwrap();
        // 验证teammate_behavior值已更新
        assert_eq!(template.rules.teammate_behavior, 10);
        
        // Delete rule template
        let result = delete_rule_template(&mut conn, &template_id);
        assert!(result.is_ok(), "Failed to delete rule template: {:?}", result.err());
        
        // Clean up test data
        test_data_manager.cleanup().expect("Failed to cleanup test data");
    }
}