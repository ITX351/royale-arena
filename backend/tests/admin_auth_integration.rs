use royale_arena_backend::{
    admin::{
        AdminService, CreateAdminRequest, LoginRequest, ResetPasswordRequest, UpdateAdminRequest,
    },
    auth::{AuthService, JwtManager},
    config::AppConfig,
};
use sqlx::mysql::MySqlPool;
use uuid::Uuid;

#[sqlx::test(migrations = "./migrations")]
async fn test_admin_auth_complete_flow(pool: MySqlPool) -> Result<(), Box<dyn std::error::Error>> {
    // 配置
    let config = AppConfig {
        database_url: "test://dummy".to_string(),
        jwt_secret: "test-secret-key-for-testing-purposes-only".to_string(),
        jwt_expiration_hours: 24,
        bcrypt_cost: 4, // 降低成本以加快测试
        server_port: 3001,
        api_prefix: "/api".to_string(),
    };

    // 创建服务
    let jwt_manager = JwtManager::new(&config.jwt_secret, config.jwt_expiration_hours);
    let auth_service = AuthService::new(pool.clone(), jwt_manager);
    let admin_service = AdminService::new(pool.clone(), config.bcrypt_cost);

    // 清理测试环境：删除所有现有管理员
    sqlx::query("DELETE FROM admin_users")
        .execute(&pool)
        .await?;

    // 测试 1: 创建测试超级管理员
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

    // 测试 2: 超级管理员登录
    let login_request = LoginRequest {
        username: "superadmin".to_string(),
        password: super_admin_password.to_string(),
    };

    let login_response_initial = auth_service.login(login_request).await?;
    assert!(login_response_initial.success);
    assert!(!login_response_initial.token.is_empty());
    assert_eq!(login_response_initial.expires_in, 24 * 3600);

    let super_admin_token = login_response_initial.token;

    // 测试 3: 验证 JWT Token
    let claims = auth_service.validate_token(&super_admin_token).await?;
    assert_eq!(claims.username, "superadmin");
    assert!(claims.is_super_admin);

    // 测试 4: 重置超级管理员密码
    let reset_request = ResetPasswordRequest {
        new_password: "superadmin_reset_456".to_string(),
    };

    let reset_response = admin_service
        .reset_password(&super_admin_id, reset_request)
        .await?;
    assert_eq!(reset_response.username, "superadmin");
    assert!(reset_response.is_super_admin);

    // 测试 5: 使用新密码登录
    let login_request_after_reset = LoginRequest {
        username: "superadmin".to_string(),
        password: "superadmin_reset_456".to_string(),
    };

    let login_response_after_reset = auth_service.login(login_request_after_reset).await?;
    assert!(login_response_after_reset.success);
    assert!(!login_response_after_reset.token.is_empty());

    // 测试 6: 创建普通管理员
    let create_request = CreateAdminRequest {
        username: "admin1".to_string(),
        password: "admin123".to_string(),
        is_super_admin: false,
    };

    let created_admin = admin_service.create_admin(create_request).await?;
    assert_eq!(created_admin.username, "admin1");
    assert!(!created_admin.is_super_admin);

    let admin1_id = created_admin.id.clone();

    // 测试 7: 获取管理员列表
    let admin_list = admin_service.list_admins().await?;
    assert_eq!(admin_list.len(), 2); // 超级管理员 + 普通管理员

    // 验证列表中包含两个用户
    let usernames: Vec<&str> = admin_list.iter().map(|u| u.username.as_str()).collect();
    assert!(usernames.contains(&"superadmin"));
    assert!(usernames.contains(&"admin1"));

    // 测试 8: 更新管理员信息
    let update_request = UpdateAdminRequest {
        username: Some("admin1_updated".to_string()),
        password: Some("newpassword123".to_string()),
        is_super_admin: None,
    };

    let updated_admin = admin_service
        .update_admin(&admin1_id, update_request)
        .await?;
    assert_eq!(updated_admin.username, "admin1_updated");
    assert!(!updated_admin.is_super_admin);

    // 测试 9: 验证密码更新后的登录
    let login_request_after_update = LoginRequest {
        username: "admin1_updated".to_string(),
        password: "newpassword123".to_string(),
    };

    let login_response_after_update = auth_service.login(login_request_after_update).await?;
    assert!(login_response_after_update.success);

    // 测试 10: 删除管理员
    admin_service.delete_admin(&admin1_id).await?;

    // 验证删除后列表只有1个用户
    let admin_list = admin_service.list_admins().await?;
    assert_eq!(admin_list.len(), 1);
    assert_eq!(admin_list[0].username, "superadmin");

    // 测试 11: 尝试删除不存在的用户（应该失败）
    let delete_result = admin_service.delete_admin("non-existent-id").await;
    assert!(delete_result.is_err());

    // 测试 12: 尝试创建重复用户名（应该失败）
    let duplicate_request = CreateAdminRequest {
        username: "superadmin".to_string(), // 重复的用户名
        password: "password123".to_string(),
        is_super_admin: false,
    };

    let duplicate_result = admin_service.create_admin(duplicate_request).await;
    assert!(duplicate_result.is_err());

    // 测试 13: 错误凭据登录（应该失败）
    let invalid_login = LoginRequest {
        username: "superadmin".to_string(),
        password: "wrongpassword".to_string(),
    };

    let invalid_result = auth_service.login(invalid_login).await;
    assert!(invalid_result.is_err());

    // 测试 14: 无效 Token 验证（应该失败）
    let invalid_token_result = auth_service.validate_token("invalid.token.here").await;
    assert!(invalid_token_result.is_err());

    println!("✅ 所有测试通过！");
    Ok(())
}
