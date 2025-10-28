//! 系统初始化模块
//! 负责系统启动时的初始化任务

use crate::admin::{AdminService, CreateAdminRequest};
use crate::database::DatabasePool;
use tracing::info;

/// 系统初始化器
pub struct SystemInitializer;

impl SystemInitializer {
    /// 执行数据库迁移，确保数据库结构与最新迁移保持一致
    pub async fn run_migrations(pool: &DatabasePool) -> Result<(), Box<dyn std::error::Error>> {
        sqlx::migrate!("./migrations").run(pool).await?;
        info!("系统初始化：数据库迁移执行完成");
        Ok(())
    }

    /// 确保至少存在一个管理员账户
    /// 当没有管理员账户时，创建一个默认的超级管理员账户
    pub async fn ensure_default_admin(
        admin_service: &AdminService,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let has_admin = admin_service
            .has_any_admin()
            .await
            .map_err(|e| -> Box<dyn std::error::Error> { Box::new(e) })?;

        if !has_admin {
            let default_admin = CreateAdminRequest {
                username: "admin".to_string(),
                password: "123456".to_string(),
                is_super_admin: true,
            };

            admin_service
                .create_admin(default_admin)
                .await
                .map(|_| ())
                .map_err(|e| -> Box<dyn std::error::Error> { Box::new(e) })?;

            info!("系统初始化：创建默认超级管理员账户 'admin'");
        } else {
            info!("系统初始化：已有管理员账户，无需创建默认账户");
        }

        Ok(())
    }

    /// 初始化系统状态
    /// 将数据库中状态为"进行中"的游戏全部变更为"暂停中"状态
    pub async fn initialize_game_states(
        pool: &DatabasePool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let result = sqlx::query("UPDATE games SET status = 'paused' WHERE status = 'running'")
            .execute(pool)
            .await?;

        let rows_affected = result.rows_affected();
        if rows_affected > 0 {
            info!(
                "系统初始化：将 {} 个进行中的游戏状态变更为暂停中",
                rows_affected
            );
        } else {
            info!("系统初始化：没有需要变更状态的游戏");
        }

        Ok(())
    }

    /// 系统初始化入口
    /// 依次执行游戏状态和管理员账户的初始化逻辑
    pub async fn initialize_system(
        pool: &DatabasePool,
        admin_service: &AdminService,
    ) -> Result<(), Box<dyn std::error::Error>> {
        Self::run_migrations(pool).await?;
        Self::initialize_game_states(pool).await?;
        Self::ensure_default_admin(admin_service).await?;

        Ok(())
    }
}
