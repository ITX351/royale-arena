#[cfg(test)]
mod director_integration_tests {
    use sqlx::MySqlPool;
    use uuid::Uuid;
    use royale_arena_backend::director::{
        DirectorService, 
        BatchAddPlayersRequest, 
        BatchDeletePlayersRequest,
        CreatePlayerRequest,
        DirectorEditGameRequest,
    };
    use royale_arena_backend::game::GameService;
    use royale_arena_backend::game::global_game_state_manager::GlobalGameStateManager;
    use royale_arena_backend::websocket::global_connection_manager::GlobalConnectionManager;
    use royale_arena_backend::routes::AppState;
    use royale_arena_backend::auth::AuthService;
    use royale_arena_backend::admin::service::AdminService;
    use royale_arena_backend::game::GameLogService;
    use royale_arena_backend::rule_template::service::RuleTemplateService;
    use royale_arena_backend::auth::JwtManager;

    #[sqlx::test(migrations = "./migrations")]
    async fn test_director_comprehensive_integration(pool: MySqlPool) -> Result<(), Box<dyn std::error::Error>> {
        // 清理相关表数据，确保测试环境干净
        sqlx::query("DELETE FROM actors").execute(&pool).await?;
        sqlx::query("DELETE FROM games").execute(&pool).await?;
        sqlx::query("DELETE FROM rule_templates").execute(&pool).await?;

        let director_service = DirectorService::new(pool.clone());

        // 创建最小的AppState用于测试
        let game_service = GameService::new(pool.clone());
        let app_state = AppState {
            auth_service: AuthService::new(pool.clone(), JwtManager::new("test_secret_key", 24)),
            admin_service: AdminService::new(pool.clone(), 10),
            director_service: director_service.clone(),
            game_service,
            game_log_service: GameLogService::new(pool.clone()),
            game_state_manager: GlobalGameStateManager::new(pool.clone()),
            rule_template_service: RuleTemplateService::new(pool.clone()),
            global_connection_manager: GlobalConnectionManager::new(),
        };

        // 创建测试数据：游戏
        let game_id = Uuid::new_v4().to_string();
        let director_password = "test123";
        sqlx::query(
            "INSERT INTO games (id, name, description, director_password, max_players, status, rules_config) 
             VALUES (?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&game_id)
        .bind("测试游戏")
        .bind("用于导演接口集成测试")
        .bind(director_password)
        .bind(100)
        .bind("waiting")
        .bind(r#"{"test": true}"#)
        .execute(&pool)
        .await?;

        // 测试1.1: 验证导演密码 - 正确密码
        let result = director_service.verify_director_password(&game_id, director_password).await;
        assert!(result.is_ok(), "正确的导演密码验证应该成功");

        // 测试1.2: 验证导演密码 - 错误密码
        let result = director_service.verify_director_password(&game_id, "wrong_password").await;
        assert!(result.is_err(), "错误的导演密码验证应该失败");

        // 测试1.3: 验证导演密码 - 不存在的游戏
        let fake_game_id = Uuid::new_v4().to_string();
        let result = director_service.verify_director_password(&fake_game_id, director_password).await;
        assert!(result.is_err(), "不存在的游戏验证应该失败");

        // 测试2.1: 批量添加演员时的密码验证 - 错误的导演密码
        let add_request_wrong_password = BatchAddPlayersRequest {
            players: vec![CreatePlayerRequest {
                player_name: "测试玩家".to_string(),
                password: "abc123".to_string(),
                team_id: Some(1),
            }],
        };

        let result = director_service
            .batch_add_players(&game_id, "wrong_password", add_request_wrong_password)
            .await;
        assert!(result.is_err(), "错误的导演密码应该导致添加操作失败");

        // 测试2.2: 批量添加演员 - 成功添加
        let add_request = BatchAddPlayersRequest {
            players: vec![
                CreatePlayerRequest {
                    player_name: "测试玩家1".to_string(),
                    password: "abc123".to_string(),
                    team_id: Some(1),
                },
                CreatePlayerRequest {
                    player_name: "测试玩家2".to_string(),
                    password: "def456".to_string(),
                    team_id: None, // 应该使用默认值0
                },
            ],
        };

        let result = director_service
            .batch_add_players(&game_id, director_password, add_request)
            .await?;

        assert_eq!(result.success.len(), 2, "应该成功添加2个演员");
        assert_eq!(result.failed.len(), 0, "不应该有失败的操作");
        assert_eq!(result.success[0].name, "测试玩家1");
        assert_eq!(result.success[0].team_id, 1);
        assert_eq!(result.success[1].name, "测试玩家2");
        assert_eq!(result.success[1].team_id, 0); // 默认值

        // 测试2.3: 批量添加演员 - 名称重复
        let duplicate_request = BatchAddPlayersRequest {
            players: vec![
                CreatePlayerRequest {
                    player_name: "测试玩家1".to_string(), // 重复名称
                    password: "xyz789".to_string(),
                    team_id: Some(2),
                },
                CreatePlayerRequest {
                    player_name: "测试玩家3".to_string(), // 新名称
                    password: "new123".to_string(),
                    team_id: Some(3),
                },
            ],
        };

        let result = director_service
            .batch_add_players(&game_id, director_password, duplicate_request)
            .await?;

        assert_eq!(result.success.len(), 1, "应该成功添加1个演员");
        assert_eq!(result.failed.len(), 1, "应该有1个失败的操作");
        assert_eq!(result.success[0].name, "测试玩家3");
        assert!(result.failed[0].reason.contains("已存在"));

        // 测试3.1: 获取演员列表时的密码验证 - 错误的导演密码
        let result = director_service.get_players(&game_id, "wrong_password").await;
        assert!(result.is_err(), "错误的导演密码应该导致获取列表失败");

        // 测试3.2: 获取演员列表 - 正确密码
        let players = director_service.get_players(&game_id, director_password).await?;
        assert_eq!(players.len(), 3, "应该有3个演员");

        // 测试3.3: 验证演员信息正确
        let player1 = players.iter().find(|p| p.name == "测试玩家1").unwrap();
        assert_eq!(player1.password, "abc123");
        assert_eq!(player1.team_id, 1);
        assert_eq!(player1.game_id, game_id);

        // 测试4.1: 游戏身份验证 - 导演密码正确（此时还没有演员，应该返回director）
        let auth_result = director_service.authenticate_game(&game_id, director_password).await?;
        assert_eq!(auth_result, "director", "使用正确的导演密码应该返回director");

        // 测试4.2: 游戏身份验证 - 导演密码错误
        let auth_result = director_service.authenticate_game(&game_id, "wrong_password").await?;
        assert_eq!(auth_result, "invalid", "使用错误的导演密码应该返回invalid");

        // 测试4.3: 游戏身份验证 - 不存在的游戏
        let auth_result = director_service.authenticate_game(&fake_game_id, director_password).await?;
        assert_eq!(auth_result, "invalid", "不存在的游戏应该返回invalid");

        // 测试4.4: 游戏身份验证 - 演员密码正确（使用刚刚添加的演员密码）
        let auth_result = director_service.authenticate_game(&game_id, "abc123").await?;
        assert_eq!(auth_result, "actor", "使用正确的演员密码应该返回actor");

        // 测试4.5: 游戏身份验证 - 演员密码错误
        let auth_result = director_service.authenticate_game(&game_id, "wrong_player_password").await?;
        assert_eq!(auth_result, "invalid", "使用错误的演员密码应该返回invalid");

        // 测试5.1: 批量删除演员时的密码验证 - 错误的导演密码
        let delete_request = BatchDeletePlayersRequest {
            player_ids: vec![Uuid::new_v4().to_string()],
        };

        let result = director_service
            .batch_delete_players(&game_id, "wrong_password", delete_request)
            .await;
        assert!(result.is_err(), "错误的导演密码应该导致删除操作失败");

        // 测试5.2: 批量删除演员 - 成功删除
        let player_ids: Vec<String> = players.iter().take(2).map(|p| p.id.clone()).collect();
        let delete_request = BatchDeletePlayersRequest {
            player_ids: player_ids.clone(),
        };

        let result = director_service
            .batch_delete_players(&game_id, director_password, delete_request)
            .await?;

        assert_eq!(result.success.len(), 2, "应该成功删除2个演员");
        assert_eq!(result.failed.len(), 0, "不应该有失败的操作");

        // 测试5.3: 验证删除后的演员列表
        let remaining_players = director_service.get_players(&game_id, director_password).await?;
        assert_eq!(remaining_players.len(), 1, "应该剩余1个演员");

        // 测试5.4: 批量删除演员 - 不存在的演员ID
        let fake_player_id = Uuid::new_v4().to_string();
        let delete_request = BatchDeletePlayersRequest {
            player_ids: vec![fake_player_id.clone()],
        };

        let result = director_service
            .batch_delete_players(&game_id, director_password, delete_request)
            .await?;

        assert_eq!(result.success.len(), 0, "不应该成功删除任何演员");
        assert_eq!(result.failed.len(), 1, "应该有1个失败的操作");
        assert!(result.failed[0].reason.contains("不存在"));

        // 测试6.1: 游戏开始后不能删除演员
        // 先修改游戏状态为运行中
        sqlx::query("UPDATE games SET status = 'running' WHERE id = ?")
            .bind(&game_id)
            .execute(&pool)
            .await?;

        let remaining_player_id = remaining_players[0].id.clone();
        let delete_request = BatchDeletePlayersRequest {
            player_ids: vec![remaining_player_id],
        };

        let result = director_service
            .batch_delete_players(&game_id, director_password, delete_request)
            .await;

        assert!(result.is_err(), "游戏运行中不应该允许删除演员");

        // 测试7.1: 数据验证 - 无效密码格式
        // 恢复游戏状态为waiting以便测试
        sqlx::query("UPDATE games SET status = 'waiting' WHERE id = ?")
            .bind(&game_id)
            .execute(&pool)
            .await?;

        let invalid_request = BatchAddPlayersRequest {
            players: vec![
                CreatePlayerRequest {
                    player_name: "测试玩家4".to_string(),
                    password: "abc@123".to_string(), // 包含特殊字符
                    team_id: Some(1),
                },
                CreatePlayerRequest {
                    player_name: "".to_string(), // 空名称
                    password: "abc123".to_string(),
                    team_id: Some(1),
                },
                CreatePlayerRequest {
                    player_name: "测试玩家5".to_string(),
                    password: "abc".to_string(), // 密码过短
                    team_id: Some(-1), // 负数队伍ID
                },
            ],
        };

        let result = director_service
            .batch_add_players(&game_id, director_password, invalid_request)
            .await?;

        assert_eq!(result.success.len(), 0, "所有添加操作都应该失败");
        assert_eq!(result.failed.len(), 3, "应该有3个失败的操作");

        // 测试7.2: 验证错误原因
        assert!(result.failed.iter().any(|f| f.reason.contains("字母和数字")));
        assert!(result.failed.iter().any(|f| f.reason.contains("不能为空")));
        assert!(result.failed.iter().any(|f| f.reason.contains("长度必须为6-8位") || f.reason.contains("不能为负数")));

        // ========== 测试8: 导演编辑游戏功能 ==========
        
        // 测试8.1: 使用错误的导演密码编辑游戏
        let edit_request = DirectorEditGameRequest {
            name: Some("新游戏名称".to_string()),
            description: None,
            max_players: None,
            rules_config: None,
        };
        let result = director_service.edit_game(&app_state, &game_id, "wrong_password", edit_request).await;
        assert!(result.is_err(), "错误的导演密码应该导致编辑失败");

        // 测试8.2: 编辑游戏名称（有效输入）
        let edit_request = DirectorEditGameRequest {
            name: Some("新游戏名称".to_string()),
            description: None,
            max_players: None,
            rules_config: None,
        };
        let result = director_service.edit_game(&app_state, &game_id, director_password, edit_request).await?;
        assert_eq!(result.name, "新游戏名称");
        assert_eq!(result.description, Some("用于导演接口集成测试".to_string()));
        assert_eq!(result.max_players, 100);

        // 测试8.3: 编辑游戏描述（有效输入）
        let edit_request = DirectorEditGameRequest {
            name: None,
            description: Some("更新后的游戏描述".to_string()),
            max_players: None,
            rules_config: None,
        };
        let result = director_service.edit_game(&app_state, &game_id, director_password, edit_request).await?;
        assert_eq!(result.description, Some("更新后的游戏描述".to_string()));

        // 测试8.4: 编辑最大玩家数（有效输入）
        let edit_request = DirectorEditGameRequest {
            name: None,
            description: None,
            max_players: Some(50),
            rules_config: None,
        };
        let result = director_service.edit_game(&app_state, &game_id, director_password, edit_request).await?;
        assert_eq!(result.max_players, 50);

        // 测试8.5: 同时编辑名称、描述和最大玩家数
        let edit_request = DirectorEditGameRequest {
            name: Some("最终游戏名称".to_string()),
            description: Some("最终描述".to_string()),
            max_players: Some(200),
            rules_config: None,
        };
        let result = director_service.edit_game(&app_state, &game_id, director_password, edit_request).await?;
        assert_eq!(result.name, "最终游戏名称");
        assert_eq!(result.description, Some("最终描述".to_string()));
        assert_eq!(result.max_players, 200);

        // 测试8.6: 编辑规则配置
        let new_rules = serde_json::json!({
            "map": {
                "places": ["地点A", "地点B"],
                "safe_places": ["地点A"]
            },
            "player": {
                "max_life": 100,
                "max_strength": 10
            }
        });
        let edit_request = DirectorEditGameRequest {
            name: None,
            description: None,
            max_players: None,
            rules_config: Some(new_rules.clone()),
        };
        let result = director_service.edit_game(&app_state, &game_id, director_password, edit_request).await?;
        assert_eq!(result.rules_config, new_rules);

        // 测试8.7: 提供空名称
        let edit_request = DirectorEditGameRequest {
            name: Some("".to_string()),
            description: None,
            max_players: None,
            rules_config: None,
        };
        let result = director_service.edit_game(&app_state, &game_id, director_password, edit_request).await;
        assert!(result.is_err(), "空名称应该导致验证错误");

        // 测试8.8: 提供超长名称（101字符）
        let edit_request = DirectorEditGameRequest {
            name: Some("a".repeat(101)),
            description: None,
            max_players: None,
            rules_config: None,
        };
        let result = director_service.edit_game(&app_state, &game_id, director_password, edit_request).await;
        assert!(result.is_err(), "超长名称应该导致验证错误");

        // 测试8.9: 提供无效的最大玩家数（0和1001）
        let edit_request = DirectorEditGameRequest {
            name: None,
            description: None,
            max_players: Some(0),
            rules_config: None,
        };
        let result = director_service.edit_game(&app_state, &game_id, director_password, edit_request).await;
        assert!(result.is_err(), "最大玩家数0应该导致验证错误");

        let edit_request = DirectorEditGameRequest {
            name: None,
            description: None,
            max_players: Some(1001),
            rules_config: None,
        };
        let result = director_service.edit_game(&app_state, &game_id, director_password, edit_request).await;
        assert!(result.is_err(), "最大玩家数1001应该导致验证错误");

        // 测试8.10: 编辑不存在的游戏
        let fake_game_id = Uuid::new_v4().to_string();
        let edit_request = DirectorEditGameRequest {
            name: Some("测试".to_string()),
            description: None,
            max_players: None,
            rules_config: None,
        };
        let result = director_service.edit_game(&app_state, &fake_game_id, director_password, edit_request).await;
        assert!(result.is_err(), "编辑不存在的游戏应该失败");

        // 测试8.11: 请求体为空（无任何字段）
        let edit_request = DirectorEditGameRequest {
            name: None,
            description: None,
            max_players: None,
            rules_config: None,
        };
        let result = director_service.edit_game(&app_state, &game_id, director_password, edit_request).await;
        assert!(result.is_err(), "空请求应该导致验证错误");

        Ok(())
    }
}