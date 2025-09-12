#[cfg(test)]
mod director_integration_tests {
    use sqlx::MySqlPool;
    use uuid::Uuid;
    use royale_arena_backend::director::{
        DirectorService, 
        BatchAddPlayersRequest, 
        BatchDeletePlayersRequest,
        CreatePlayerRequest,
    };

    #[sqlx::test(migrations = "./migrations")]
    async fn test_director_comprehensive_integration(pool: MySqlPool) -> Result<(), Box<dyn std::error::Error>> {
        // 清理相关表数据，确保测试环境干净
        sqlx::query("DELETE FROM actors").execute(&pool).await?;
        sqlx::query("DELETE FROM games").execute(&pool).await?;
        sqlx::query("DELETE FROM rule_templates").execute(&pool).await?;

        let director_service = DirectorService::new(pool.clone());

        // 创建测试数据：规则模板
        let rule_template_id = Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO rule_templates (id, template_name, description, is_active, rules_config) 
             VALUES (?, ?, ?, ?, ?)"
        )
        .bind(&rule_template_id)
        .bind("测试规则模板")
        .bind("用于集成测试")
        .bind(true)
        .bind(r#"{"test": true}"#)
        .execute(&pool)
        .await?;

        // 创建测试数据：游戏
        let game_id = Uuid::new_v4().to_string();
        let director_password = "test123";
        sqlx::query(
            "INSERT INTO games (id, name, description, director_password, max_players, status, rule_template_id) 
             VALUES (?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&game_id)
        .bind("测试游戏")
        .bind("用于导演接口集成测试")
        .bind(director_password)
        .bind(100)
        .bind("waiting")
        .bind(&rule_template_id)
        .execute(&pool)
        .await?;

        // 测试1: 验证导演密码 - 正确密码
        let result = director_service.verify_director_password(&game_id, director_password).await;
        assert!(result.is_ok(), "正确的导演密码验证应该成功");

        // 测试2: 验证导演密码 - 错误密码
        let result = director_service.verify_director_password(&game_id, "wrong_password").await;
        assert!(result.is_err(), "错误的导演密码验证应该失败");

        // 测试3: 验证导演密码 - 不存在的游戏
        let fake_game_id = Uuid::new_v4().to_string();
        let result = director_service.verify_director_password(&fake_game_id, director_password).await;
        assert!(result.is_err(), "不存在的游戏验证应该失败");

        // 测试4: 批量添加演员时的密码验证 - 错误的导演密码
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

        // 测试5: 批量添加演员 - 成功添加
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

        // 测试6: 批量添加演员 - 名称重复
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

        // 测试7: 获取演员列表时的密码验证 - 错误的导演密码
        let result = director_service.get_players(&game_id, "wrong_password").await;
        assert!(result.is_err(), "错误的导演密码应该导致获取列表失败");

        // 测试8: 获取演员列表 - 正确密码
        let players = director_service.get_players(&game_id, director_password).await?;
        assert_eq!(players.len(), 3, "应该有3个演员");

        // 验证演员信息正确
        let player1 = players.iter().find(|p| p.name == "测试玩家1").unwrap();
        assert_eq!(player1.password, "abc123");
        assert_eq!(player1.team_id, 1);
        assert_eq!(player1.game_id, game_id);

        let player2 = players.iter().find(|p| p.name == "测试玩家2").unwrap();
        assert_eq!(player2.password, "def456");
        assert_eq!(player2.team_id, 0);

        // 测试9: 批量删除演员时的密码验证 - 错误的导演密码
        let delete_request = BatchDeletePlayersRequest {
            player_ids: vec![Uuid::new_v4().to_string()],
        };

        let result = director_service
            .batch_delete_players(&game_id, "wrong_password", delete_request)
            .await;
        assert!(result.is_err(), "错误的导演密码应该导致删除操作失败");

        // 测试10: 批量删除演员 - 成功删除
        let player_ids: Vec<String> = players.iter().take(2).map(|p| p.id.clone()).collect();
        let delete_request = BatchDeletePlayersRequest {
            player_ids: player_ids.clone(),
        };

        let result = director_service
            .batch_delete_players(&game_id, director_password, delete_request)
            .await?;

        assert_eq!(result.success.len(), 2, "应该成功删除2个演员");
        assert_eq!(result.failed.len(), 0, "不应该有失败的操作");

        // 验证删除后的演员列表
        let remaining_players = director_service.get_players(&game_id, director_password).await?;
        assert_eq!(remaining_players.len(), 1, "应该剩余1个演员");

        // 测试11: 批量删除演员 - 不存在的演员ID
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

        // 测试12: 游戏开始后不能删除演员
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

        // 测试13: 数据验证 - 无效密码格式
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

        // 验证错误原因
        assert!(result.failed.iter().any(|f| f.reason.contains("字母和数字")));
        assert!(result.failed.iter().any(|f| f.reason.contains("不能为空")));
        assert!(result.failed.iter().any(|f| f.reason.contains("长度必须为6-8位") || f.reason.contains("不能为负数")));

        Ok(())
    }
}