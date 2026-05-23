use royale_arena_backend::{
    admin::{AdminService, LoginRequest},
    auth::{AuthService, JwtManager},
    config::AppConfig,
    rule_template::{CreateRuleTemplateRequest, RuleTemplateService, UpdateRuleTemplateRequest},
};
use serde_json::json;
use sqlx::mysql::MySqlPool;
use uuid::Uuid;

#[sqlx::test(migrations = "./migrations")]
async fn test_rule_template_complete_flow(
    pool: MySqlPool,
) -> Result<(), Box<dyn std::error::Error>> {
    // 清理测试环境
    sqlx::query("DELETE FROM rule_templates WHERE template_name LIKE '%集成测试%'")
        .execute(&pool)
        .await?;

    sqlx::query("DELETE FROM admin_users")
        .execute(&pool)
        .await?;

    // 配置
    let config = AppConfig {
        database_url: "test://dummy".to_string(),
        jwt_secret: "test-secret-key-for-testing-purposes-only".to_string(),
        jwt_expiration_hours: 24,
        bcrypt_cost: 4, // 降低成本以加快测试
        server_port: 3000,
        api_prefix: "/api".to_string(),
    };

    // 创建服务
    let jwt_manager = JwtManager::new(&config.jwt_secret, config.jwt_expiration_hours);
    let auth_service = AuthService::new(pool.clone(), jwt_manager);
    let _admin_service = AdminService::new(pool.clone(), config.bcrypt_cost);
    let rule_template_service = RuleTemplateService::new(pool.clone());

    // 准备测试环境：创建管理员账户
    let super_admin_id = Uuid::new_v4().to_string();
    let super_admin_password = "superadmin123";
    let hashed_password = bcrypt::hash(super_admin_password, config.bcrypt_cost)?;

    sqlx::query(
        r#"
        INSERT INTO admin_users (id, username, password, is_super_admin)
        VALUES (?, ?, ?, ?)
        "#,
    )
    .bind(&super_admin_id)
    .bind("superadmin")
    .bind(&hashed_password)
    .bind(true)
    .execute(&pool)
    .await?;

    // 管理员登录获取token
    let login_request = LoginRequest {
        username: "superadmin".to_string(),
        password: super_admin_password.to_string(),
    };

    let login_response = auth_service.login(login_request).await?;
    let _admin_token = login_response.token;

    // 测试数据
    let test_rules_config = json!({
        "game_flow": {
            "day_duration": 300,
            "night_duration": 900
        },
        "map": {
            "places": [
                "码头", "工厂", "贫民窟", "旅馆", "教堂", "市政厅", "消防局", "池塘",
                "住宅区", "灯塔", "小巷", "学校", "隧道", "山道", "寺庙", "靶场",
                "医院", "森林", "海滩", "墓园", "井", "研究中心"
            ]
        },
        "player": {
            "max_life": 100,
            "max_strength": 100,
            "daily_life_recovery": 0,
            "daily_strength_recovery": 40
        },
        "action": {
            "move_cost": 5,
            "search_cost": 5,
            "search_cooldown": 30
        },
        "rest_mode": {
            "life_recovery": 25,
            "strength_recovery": 1000,
            "max_moves": 1
        },
        "teammate_behavior": 0
    });

    // 测试 1: 创建规则模版
    let create_request = CreateRuleTemplateRequest {
        template_name: "集成测试经典模版".to_string(),
        description: Some("用于集成测试的经典大逃杀规则模版".to_string()),
        is_active: Some(true),
        rules_config: test_rules_config.clone(),
    };

    let created_template = rule_template_service
        .create_template(create_request)
        .await?;
    assert_eq!(created_template.template_name, "集成测试经典模版");
    assert_eq!(
        created_template.description,
        Some("用于集成测试的经典大逃杀规则模版".to_string())
    );
    assert!(created_template.is_active);
    assert_eq!(created_template.rules_config, test_rules_config);

    let template_id = created_template.id.clone();

    // 测试 2: 创建第二个模版用于列表测试
    let create_request2 = CreateRuleTemplateRequest {
        template_name: "集成测试快速模版".to_string(),
        description: Some("快速游戏模式".to_string()),
        is_active: Some(false),
        rules_config: json!({
            "game_flow": {
                "day_duration": 180,
                "night_duration": 540
            },
            "player": {
                "max_life": 80,
                "max_strength": 80
            }
        }),
    };

    let created_template2 = rule_template_service
        .create_template(create_request2)
        .await?;
    let template_id2 = created_template2.id.clone();

    // 测试 3: 获取所有模版列表
    let all_templates = rule_template_service
        .get_templates(None, None, None)
        .await?;
    let test_templates: Vec<_> = all_templates
        .iter()
        .filter(|t| t.template_name.contains("集成测试"))
        .collect();
    assert_eq!(test_templates.len(), 2);

    // 测试 4: 根据ID获取特定模版
    let specific_template = rule_template_service
        .get_templates(Some(template_id.clone()), None, None)
        .await?;
    assert_eq!(specific_template.len(), 1);
    assert_eq!(specific_template[0].template_name, "集成测试经典模版");
    assert_eq!(specific_template[0].rules_config, test_rules_config);

    // 测试 5: 根据激活状态筛选
    let active_templates = rule_template_service
        .get_templates(None, Some(true), None)
        .await?;
    let active_test_templates: Vec<_> = active_templates
        .iter()
        .filter(|t| t.template_name.contains("集成测试") && t.is_active)
        .collect();
    assert_eq!(active_test_templates.len(), 1);
    assert_eq!(active_test_templates[0].template_name, "集成测试经典模版");

    let inactive_templates = rule_template_service
        .get_templates(None, Some(false), None)
        .await?;
    let inactive_test_templates: Vec<_> = inactive_templates
        .iter()
        .filter(|t| t.template_name.contains("集成测试") && !t.is_active)
        .collect();
    assert_eq!(inactive_test_templates.len(), 1);
    assert_eq!(inactive_test_templates[0].template_name, "集成测试快速模版");

    // 测试 6: 根据名称搜索
    let search_templates = rule_template_service
        .get_templates(None, None, Some("经典".to_string()))
        .await?;
    let search_test_templates: Vec<_> = search_templates
        .iter()
        .filter(|t| t.template_name.contains("集成测试"))
        .collect();
    assert_eq!(search_test_templates.len(), 1);
    assert_eq!(search_test_templates[0].template_name, "集成测试经典模版");

    // 测试 7: 更新规则模版
    let updated_rules_config = json!({
        "game_flow": {
            "day_duration": 400,
            "night_duration": 1000
        },
        "map": {
            "places": ["码头", "工厂", "学校"]
        },
        "player": {
            "max_life": 120,
            "max_strength": 120,
            "daily_life_recovery": 3,
            "daily_strength_recovery": 50
        },
        "rest_mode": {
            "life_recovery": 30,
            "strength_recovery": 1200,
            "max_moves": 2
        }
    });

    let update_request = UpdateRuleTemplateRequest {
        template_name: Some("集成测试经典模版v2".to_string()),
        description: Some("更新后的经典模版描述".to_string()),
        is_active: Some(false),
        rules_config: Some(updated_rules_config.clone()),
    };

    let updated_template = rule_template_service
        .update_template(template_id.clone(), update_request)
        .await?;
    assert_eq!(updated_template.template_name, "集成测试经典模版v2");
    assert_eq!(
        updated_template.description,
        Some("更新后的经典模版描述".to_string())
    );
    assert!(!updated_template.is_active);
    assert_eq!(updated_template.rules_config, updated_rules_config);

    // 测试 8: 部分更新模版（只更新名称）
    let partial_update_request = UpdateRuleTemplateRequest {
        template_name: Some("集成测试经典模版v3".to_string()),
        description: None,
        is_active: None,
        rules_config: None,
    };

    let partial_updated_template = rule_template_service
        .update_template(template_id.clone(), partial_update_request)
        .await?;
    assert_eq!(partial_updated_template.template_name, "集成测试经典模版v3");
    assert_eq!(
        partial_updated_template.description,
        Some("更新后的经典模版描述".to_string())
    ); // 保持不变
    assert!(!partial_updated_template.is_active); // 保持不变
    assert_eq!(partial_updated_template.rules_config, updated_rules_config); // 保持不变

    // 测试 9: 错误场景测试

    // 9.1: 创建重复名称模版（应该失败）
    let duplicate_request = CreateRuleTemplateRequest {
        template_name: "集成测试经典模版v3".to_string(), // 重复名称
        description: Some("重复模版".to_string()),
        is_active: Some(true),
        rules_config: json!({"test": true}),
    };

    let duplicate_result = rule_template_service
        .create_template(duplicate_request)
        .await;
    assert!(duplicate_result.is_err());

    // 9.2: 更新不存在的模版（应该失败）
    let nonexistent_update = UpdateRuleTemplateRequest {
        template_name: Some("不存在的模版".to_string()),
        description: None,
        is_active: None,
        rules_config: None,
    };

    let nonexistent_result = rule_template_service
        .update_template("nonexistent-id".to_string(), nonexistent_update)
        .await;
    assert!(nonexistent_result.is_err());

    // 9.3: 创建空名称模版（应该失败）
    let empty_name_request = CreateRuleTemplateRequest {
        template_name: "".to_string(),
        description: None,
        is_active: Some(true),
        rules_config: json!({"test": true}),
    };

    let empty_name_result = rule_template_service
        .create_template(empty_name_request)
        .await;
    assert!(empty_name_result.is_err());

    // 9.4: 创建过长名称模版（应该失败）
    let long_name_request = CreateRuleTemplateRequest {
        template_name: "a".repeat(101), // 超过100字符限制
        description: None,
        is_active: Some(true),
        rules_config: json!({"test": true}),
    };

    let long_name_result = rule_template_service
        .create_template(long_name_request)
        .await;
    assert!(long_name_result.is_err());

    // 9.5: 创建无效JSON配置模版（应该失败）
    let invalid_config_request = CreateRuleTemplateRequest {
        template_name: "集成测试无效配置".to_string(),
        description: None,
        is_active: Some(true),
        rules_config: json!("not an object"), // 不是对象
    };

    let invalid_config_result = rule_template_service
        .create_template(invalid_config_request)
        .await;
    assert!(invalid_config_result.is_err());

    // 9.6: 更新时使用重复名称（应该失败）
    let duplicate_name_update = UpdateRuleTemplateRequest {
        template_name: Some("集成测试快速模版".to_string()), // 已存在的名称
        description: None,
        is_active: None,
        rules_config: None,
    };

    let duplicate_name_result = rule_template_service
        .update_template(template_id.clone(), duplicate_name_update)
        .await;
    assert!(duplicate_name_result.is_err());

    // 9.7: 空更新请求（应该失败）
    let empty_update_request = UpdateRuleTemplateRequest {
        template_name: None,
        description: None,
        is_active: None,
        rules_config: None,
    };

    let empty_update_result = rule_template_service
        .update_template(template_id.clone(), empty_update_request)
        .await;
    assert!(empty_update_result.is_err());

    // 测试 10: 验证最终状态
    let final_templates = rule_template_service
        .get_templates(None, None, None)
        .await?;
    let final_test_templates: Vec<_> = final_templates
        .iter()
        .filter(|t| t.template_name.contains("集成测试"))
        .collect();
    assert_eq!(final_test_templates.len(), 2);

    // 验证模版内容
    let updated_template_final = rule_template_service
        .get_templates(Some(template_id), None, None)
        .await?;
    assert_eq!(updated_template_final.len(), 1);
    assert_eq!(
        updated_template_final[0].template_name,
        "集成测试经典模版v3"
    );

    let second_template_final = rule_template_service
        .get_templates(Some(template_id2.clone()), None, None)
        .await?;
    assert_eq!(second_template_final.len(), 1);
    assert_eq!(second_template_final[0].template_name, "集成测试快速模版");

    // 测试 11: 删除规则模版
    rule_template_service
        .delete_template(template_id2.clone())
        .await?;

    let after_delete_templates = rule_template_service
        .get_templates(None, None, None)
        .await?;
    let after_delete_test_templates: Vec<_> = after_delete_templates
        .iter()
        .filter(|t| t.template_name.contains("集成测试"))
        .collect();
    assert_eq!(after_delete_test_templates.len(), 1);

    // 删除后的模版不应再能查询到
    let deleted_template = rule_template_service
        .get_templates(Some(template_id2.clone()), None, None)
        .await?;
    assert!(deleted_template.is_empty());

    // 删除不存在的模版应返回错误
    let delete_nonexistent_result = rule_template_service
        .delete_template(template_id2.clone())
        .await;
    assert!(delete_nonexistent_result.is_err());

    println!("✅ 规则模版集成测试全部通过！");

    // === 服务层测试 ===
    // 清理测试数据
    sqlx::query("DELETE FROM rule_templates WHERE template_name LIKE '%服务层测试%'")
        .execute(&pool)
        .await?;

    // 测试服务层CRUD操作
    let service_test_request = CreateRuleTemplateRequest {
        template_name: "服务层测试模版".to_string(),
        description: Some("这是一个服务层测试模版".to_string()),
        is_active: Some(true),
        rules_config: json!({
            "game_flow": {
                "day_duration": 300,
                "night_duration": 900
            },
            "player": {
                "max_life": 100,
                "max_strength": 100
            }
        }),
    };

    let service_created = rule_template_service
        .create_template(service_test_request)
        .await?;
    assert_eq!(service_created.template_name, "服务层测试模版");
    assert_eq!(
        service_created.description,
        Some("这是一个服务层测试模版".to_string())
    );
    assert!(service_created.is_active);
    let service_template_id = service_created.id.clone();

    // 测试更新服务
    let service_update_request = UpdateRuleTemplateRequest {
        template_name: Some("更新的服务层测试模版".to_string()),
        description: Some("更新后的描述".to_string()),
        is_active: Some(false),
        rules_config: Some(json!({
            "game_flow": {
                "day_duration": 600,
                "night_duration": 1200
            }
        })),
    };

    let service_updated = rule_template_service
        .update_template(service_created.id.clone(), service_update_request)
        .await?;
    assert_eq!(service_updated.template_name, "更新的服务层测试模版");
    assert_eq!(
        service_updated.description,
        Some("更新后的描述".to_string())
    );
    assert!(!service_updated.is_active);

    // 测试名称唯一性
    let duplicate_request = CreateRuleTemplateRequest {
        template_name: "更新的服务层测试模版".to_string(),
        description: None,
        is_active: Some(true),
        rules_config: json!({"test": true}),
    };
    let duplicate_result = rule_template_service
        .create_template(duplicate_request)
        .await;
    assert!(duplicate_result.is_err());

    // 测试更新不存在的模版
    let nonexistent_update = UpdateRuleTemplateRequest {
        template_name: Some("不存在的模版".to_string()),
        description: None,
        is_active: None,
        rules_config: None,
    };
    let nonexistent_result = rule_template_service
        .update_template("non-existent-id".to_string(), nonexistent_update)
        .await;
    assert!(nonexistent_result.is_err());

    // 测试删除服务层模版
    rule_template_service
        .delete_template(service_template_id.clone())
        .await?;

    let deleted_service_template = rule_template_service
        .get_templates(Some(service_template_id.clone()), None, None)
        .await?;
    assert!(deleted_service_template.is_empty());

    let delete_service_again = rule_template_service
        .delete_template(service_template_id)
        .await;
    assert!(delete_service_again.is_err());

    println!("✅ 服务层测试全部通过！");
    Ok(())
}
