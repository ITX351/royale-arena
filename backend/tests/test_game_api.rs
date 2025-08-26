use royale_arena_backend::{
    game::{CreateGameRequest, GameService, UpdateGameRequest, GameListQuery, GameStatus, GameFilterType},
};
use serde_json::json;
use sqlx::MySqlPool;
use uuid::Uuid;

#[sqlx::test(migrations = "./migrations")]
async fn test_game_crud_operations(pool: MySqlPool) -> Result<(), Box<dyn std::error::Error>> {
    // 清理测试数据
    sqlx::query("DELETE FROM games WHERE name LIKE 'test_%'")
        .execute(&pool).await?;
    sqlx::query("DELETE FROM rule_templates WHERE template_name LIKE 'test_%'")
        .execute(&pool).await?;
    
    let service = GameService::new(pool.clone());
    
    // 测试1: 创建游戏（无规则模板）
    let create_request = CreateGameRequest {
        name: "test_game_1".to_string(),
        description: Some("测试游戏描述".to_string()),
        director_password: "password123".to_string(),
        max_players: 10,
        rule_template_id: None,
    };
    
    let created_game = service.create_game(create_request).await?;
    assert_eq!(created_game.name, "test_game_1");
    assert_eq!(created_game.max_players, 10);
    assert_eq!(created_game.status, GameStatus::Waiting);
    assert!(created_game.rule_template_id.is_none());
    
    // 测试2: 创建规则模板并用于游戏
    let template_id = Uuid::new_v4().to_string();
    sqlx::query!(
        r#"
        INSERT INTO rule_templates (id, template_name, description, is_active, rules_config)
        VALUES (?, ?, ?, TRUE, ?)
        "#,
        template_id,
        "test_template_1",
        "测试规则模板",
        json!({"turn_time": 300, "max_rounds": 10})
    ).execute(&pool).await?;
    
    let create_request_with_template = CreateGameRequest {
        name: "test_game_2".to_string(),
        description: Some("带规则模板的测试游戏".to_string()),
        director_password: "password456".to_string(),
        max_players: 20,
        rule_template_id: Some(template_id.clone()),
    };
    
    let game_with_template = service.create_game(create_request_with_template).await?;
    assert_eq!(game_with_template.name, "test_game_2");
    assert_eq!(game_with_template.rule_template_id, Some(template_id.clone()));
    
    // 测试3: 更新游戏
    let update_request = UpdateGameRequest {
        name: Some("updated_game_name".to_string()),
        description: Some("更新后的描述".to_string()),
        director_password: Some("newpassword".to_string()),
        max_players: Some(15),
        rule_template_id: None,
    };
    
    let updated_game = service.update_game(&created_game.id, update_request).await?;
    assert_eq!(updated_game.name, "updated_game_name");
    assert_eq!(updated_game.max_players, 15);
    assert!(updated_game.rule_template_id.is_none());
    
    // 测试4: 获取游戏列表
    let all_games_query = GameListQuery { filter: None };
    let all_games = service.get_games(&all_games_query, false).await?;
    assert!(all_games.len() >= 2);
    
    let waiting_games_query = GameListQuery { filter: Some(GameFilterType::Waiting) };
    let waiting_games = service.get_games(&waiting_games_query, false).await?;
    assert!(waiting_games.len() >= 2);
    
    // 测试4.5: 验证游戏列表的权限控制行为
    // 检查没有管理员权限时，游戏列表项中不包含导演密码
    for game in &all_games {
        assert!(game.director_password.is_none(), "Non-admin user should not see director password in game list");
    }
    
    // 检查有管理员权限时，游戏列表项中包含导演密码
    let all_games_with_password = service.get_games(&all_games_query, true).await?;
    assert!(all_games_with_password.len() >= 2);
    
    // 验证至少有一个游戏包含导演密码
    let game_with_password = all_games_with_password.iter().find(|g| g.director_password.is_some());
    assert!(game_with_password.is_some(), "Admin user should see director password in game list");
    
    // 验证密码内容正确
    if let Some(game) = game_with_password {
        // 根据创建请求中的密码验证
        if game.name == "test_game_1" {
            assert_eq!(game.director_password.as_ref().unwrap(), "password123");
        } else if game.name == "test_game_2" {
            assert_eq!(game.director_password.as_ref().unwrap(), "password456");
        }
    }
    
    // 测试5: 获取游戏详情（包含规则）
    let game_details = service.get_game_with_rules(&game_with_template.id, false).await?;
    assert_eq!(game_details.name, "test_game_2");
    assert!(game_details.rule_template.is_some());
    
    let rule_template = game_details.rule_template.unwrap();
    assert_eq!(rule_template.template_name, "test_template_1");
    assert_eq!(rule_template.id, template_id);
    
    let game_without_rules = service.get_game_with_rules(&updated_game.id, false).await?;
    assert_eq!(game_without_rules.name, "updated_game_name");
    assert!(game_without_rules.rule_template.is_none());
    
    // 测试5.5: 验证权限控制行为
    // 没有管理员权限时，不应该返回导演密码
    assert!(game_details.director_password.is_none());
    assert!(game_without_rules.director_password.is_none());
    
    // 有管理员权限时，应该返回导演密码
    let game_with_password = service.get_game_with_rules(&game_with_template.id, true).await?;
    assert!(game_with_password.director_password.is_some());
    assert_eq!(game_with_password.director_password.unwrap(), "password456");
    
    // 测试6: 删除游戏
    service.delete_game(&created_game.id).await?;
    
    // 验证游戏已删除
    let result = service.get_game_with_rules(&created_game.id, false).await;
    assert!(result.is_err());
    
    // 测试7: 验证错误情况
    
    // 重复名称检查
    let duplicate_name_request = CreateGameRequest {
        name: "test_game_2".to_string(), // 与已存在的游戏同名
        description: None,
        director_password: "password".to_string(),
        max_players: 5,
        rule_template_id: None,
    };
    let duplicate_result = service.create_game(duplicate_name_request).await;
    assert!(duplicate_result.is_err());
    
    // 无效规则模板ID
    let invalid_template_request = CreateGameRequest {
        name: "test_game_3".to_string(),
        description: None,
        director_password: "password".to_string(),
        max_players: 5,
        rule_template_id: Some("non-existent-template".to_string()),
    };
    let invalid_template_result = service.create_game(invalid_template_request).await;
    assert!(invalid_template_result.is_err());
    
    // 更新不存在的游戏
    let update_nonexistent = UpdateGameRequest {
        name: Some("new_name".to_string()),
        description: None,
        director_password: None,
        max_players: None,
        rule_template_id: None,
    };
    let update_result = service.update_game("non-existent-id", update_nonexistent).await;
    assert!(update_result.is_err());
    
    // 删除不存在的游戏
    let delete_result = service.delete_game("non-existent-id").await;
    assert!(delete_result.is_err());
    
    // 清理测试数据
    sqlx::query("DELETE FROM games WHERE name LIKE 'test_%' OR name LIKE 'updated_%'")
        .execute(&pool).await?;
    sqlx::query("DELETE FROM rule_templates WHERE template_name LIKE 'test_%'")
        .execute(&pool).await?;
    
    Ok(())
}

#[sqlx::test(migrations = "./migrations")]
async fn test_game_validation(pool: MySqlPool) -> Result<(), Box<dyn std::error::Error>> {
    let service = GameService::new(pool);
    
    // 测试参数验证
    
    // 空游戏名称
    let empty_name_request = CreateGameRequest {
        name: "".to_string(),
        description: None,
        director_password: "password123".to_string(),
        max_players: 10,
        rule_template_id: None,
    };
    let result = service.create_game(empty_name_request).await;
    assert!(result.is_err());
    
    // 游戏名称过长
    let long_name_request = CreateGameRequest {
        name: "a".repeat(101),
        description: None,
        director_password: "password123".to_string(),
        max_players: 10,
        rule_template_id: None,
    };
    let result = service.create_game(long_name_request).await;
    assert!(result.is_err());
    
    // 密码过短
    let short_password_request = CreateGameRequest {
        name: "valid_name".to_string(),
        description: None,
        director_password: "123".to_string(),
        max_players: 10,
        rule_template_id: None,
    };
    let result = service.create_game(short_password_request).await;
    assert!(result.is_err());
    
    // 密码过长
    let long_password_request = CreateGameRequest {
        name: "valid_name".to_string(),
        description: None,
        director_password: "a".repeat(51),
        max_players: 10,
        rule_template_id: None,
    };
    let result = service.create_game(long_password_request).await;
    assert!(result.is_err());
    
    // 玩家数量无效
    let invalid_players_request = CreateGameRequest {
        name: "valid_name".to_string(),
        description: None,
        director_password: "password123".to_string(),
        max_players: 0, // 无效值
        rule_template_id: None,
    };
    let result = service.create_game(invalid_players_request).await;
    assert!(result.is_err());
    
    let too_many_players_request = CreateGameRequest {
        name: "valid_name".to_string(),
        description: None,
        director_password: "password123".to_string(),
        max_players: 1001, // 超过限制
        rule_template_id: None,
    };
    let result = service.create_game(too_many_players_request).await;
    assert!(result.is_err());
    
    Ok(())
}