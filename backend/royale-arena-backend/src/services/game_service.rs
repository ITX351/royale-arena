use crate::models::game::Game;
use crate::models::rules::GameRules;
use crate::services::db_helper::get_db_connection_from_pool;
use actix_web::error::ErrorInternalServerError;
use mysql::prelude::*;

/// 从数据库获取游戏信息
pub fn get_game_from_db(game_id: &str) -> Result<Option<Game>, actix_web::Error> {
    let mut conn = get_db_connection_from_pool()?;
    
    // 使用Vec<Value>来避免参数数量限制
    let params = vec![game_id.into()];
    
    let result: Result<Option<mysql::Row>, _> = 
        conn.exec_first(
            r"SELECT id, name, description, status, rule_template_id, phase, player_count, max_players, 
                     start_time, end_time, action_start_time, action_end_time, safe_zones, 
                     weather, announcements
              FROM games WHERE id = ?",
            mysql::Params::Positional(params)
        );
    
    // Log the result for debugging
    match &result {
        Ok(_) => tracing::debug!("Database query successful"),
        Err(e) => tracing::error!("Database query failed: {}", e),
    }
    
    let result = result.map_err(|e| {
        tracing::error!("Failed to query game from database: {}", e);
        ErrorInternalServerError("Database query error")
    })?;
    
    match result {
        Some(row) => {
            let id: String = row.get(0).unwrap_or_default();
            let name: String = row.get(1).unwrap_or_default();
            let description: String = row.get(2).unwrap_or_default();
            let status: String = row.get(3).unwrap_or_default();
            let rule_template_id: Option<String> = row.get(4).unwrap_or_default();
            let phase: String = row.get(5).unwrap_or_default();
            let player_count: u32 = row.get(6).unwrap_or_default();
            let max_players: u32 = row.get(7).unwrap_or_default();
            let start_time: Option<String> = row.get(8).unwrap_or_default();
            let end_time: Option<String> = row.get(9).unwrap_or_default();
            let action_start_time: Option<String> = row.get(10).unwrap_or_default();
            let action_end_time: Option<String> = row.get(11).unwrap_or_default();
            let safe_zones_json: String = row.get(12).unwrap_or_default();
            let weather: f64 = row.get(13).unwrap_or_default();
            let announcements_json: String = row.get(14).unwrap_or_default();
            
            // 解析JSON字段
            let safe_zones: Vec<String> = if !safe_zones_json.is_empty() {
                serde_json::from_str(&safe_zones_json).map_err(|e| {
                    tracing::error!("Failed to parse safe_zones JSON: {}", e);
                    ErrorInternalServerError("Data parsing error")
                })?
            } else {
                Vec::new()
            };
            
            let announcements: Vec<String> = if !announcements_json.is_empty() {
                serde_json::from_str(&announcements_json).map_err(|e| {
                    tracing::error!("Failed to parse announcements JSON: {}", e);
                    ErrorInternalServerError("Data parsing error")
                })?
            } else {
                Vec::new()
            };
            
            Ok(Some(Game {
                id,
                name,
                description,
                status,
                rule_template_id,
                phase,
                player_count,
                max_players,
                start_time,
                end_time,
                action_start_time,
                action_end_time,
                safe_zones,
                weather,
                announcements,
            }))
        },
        None => Ok(None)
    }
}

/// 从数据库获取规则模板
pub fn get_rule_template_from_db(template_id: &str) -> Result<Option<GameRules>, actix_web::Error> {
    let mut conn = get_db_connection_from_pool()?;
    
    let result: Option<(Option<i32>, Option<i32>, Option<i32>, Option<i32>, Option<i32>, 
                        Option<i32>, Option<i32>, Option<i32>, Option<i32>, 
                        Option<String>, Option<i32>)> = 
        conn.exec_first(
            r"SELECT max_life, max_strength, daily_strength_recovery, life_recovery,
                     move_cost, search_cost, search_cooldown, max_moves, day_duration, 
                     places, teammate_behavior
              FROM rule_templates WHERE id = ?",
            (template_id,)
        ).map_err(|e| {
            tracing::error!("Failed to query rule template from database: {}", e);
            ErrorInternalServerError("Database query error")
        })?;
    
    match result {
        Some((max_life, max_strength, daily_strength_recovery, life_recovery,
              move_cost, search_cost, search_cooldown, max_moves, day_duration, 
              places_json, teammate_behavior)) => {
            
            // 解析地点JSON
            let places: Vec<String> = if let Some(places_str) = places_json {
                if !places_str.is_empty() {
                    serde_json::from_str(&places_str).map_err(|e| {
                        tracing::error!("Failed to parse places JSON: {}", e);
                        ErrorInternalServerError("Data parsing error")
                    })?
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            };
            
            // 创建GameRules对象
            let mut rules = GameRules::default();
            
            if let Some(val) = max_life { rules.max_life = val as u32; }
            if let Some(val) = max_strength { rules.max_strength = val as u32; }
            if let Some(val) = daily_strength_recovery { rules.day_recovery = val as u32; }
            if let Some(val) = life_recovery { rules.rest_recovery = val as u32; }
            if let Some(val) = move_cost { rules.move_cost = val as u32; }
            if let Some(val) = search_cost { rules.search_cost = val as u32; }
            if let Some(val) = search_cooldown { rules.search_interval = val as u32; }
            if let Some(val) = max_moves { rules.rest_move_limit = val as u32; }
            if let Some(val) = day_duration { rules.game_duration = val as u32; }
            if let Some(val) = teammate_behavior { rules.teammate_behavior = val; }
            if !places.is_empty() { rules.places = places; }
            
            Ok(Some(rules))
        },
        None => Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use crate::models::game::Game;

    #[test]
    fn test_placeholder() {
        // Placeholder test - will be replaced when actual implementation is added
        assert!(true);
    }
    
    #[test]
    fn test_game_struct_serialization() {
        // 测试Game结构体的序列化是否包含新的rule_template_id字段
        let game = Game {
            id: "test".to_string(),
            name: "Test Game".to_string(),
            description: "A test game".to_string(),
            status: "waiting".to_string(),
            rule_template_id: Some("template1".to_string()),
            phase: "day".to_string(),
            player_count: 0,
            max_players: 100,
            start_time: None,
            end_time: None,
            action_start_time: None,
            action_end_time: None,
            safe_zones: vec![],
            weather: 0.0,
            announcements: vec![],
        };
        
        let json = serde_json::to_string(&game).unwrap();
        assert!(json.contains("\"rule_template_id\":\"template1\""));
        
        // 测试反序列化
        let deserialized: Game = serde_json::from_str(&json).unwrap();
        assert_eq!(game.rule_template_id, deserialized.rule_template_id);
    }
    
    #[test]
    fn test_game_rules_serialization() {
        // 测试GameRules序列化是否包含队友行为规则
        let mut rules = crate::models::rules::GameRules::default();
        rules.teammate_behavior = 10; // 禁止搜索 + 允许赠送
        
        let json = serde_json::to_string(&rules).unwrap();
        assert!(json.contains("\"teammate_behavior\":10"));
        
        // 测试反序列化
        let deserialized: crate::models::rules::GameRules = serde_json::from_str(&json).unwrap();
        assert_eq!(rules.teammate_behavior, deserialized.teammate_behavior);
    }
}