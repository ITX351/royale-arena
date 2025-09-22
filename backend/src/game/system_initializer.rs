//! 系统初始化模块
//! 负责系统启动时的初始化任务

use sqlx::MySqlPool;
use tracing::info;

/// 系统初始化器
pub struct SystemInitializer;

impl SystemInitializer {
    /// 初始化系统状态
    /// 将数据库中状态为"进行中"的游戏全部变更为"暂停中"状态
    pub async fn initialize_game_states(pool: &MySqlPool) -> Result<(), Box<dyn std::error::Error>> {
        let result = sqlx::query(
            "UPDATE games SET status = 'paused' WHERE status = 'running'"
        )
        .execute(pool)
        .await?;

        let rows_affected = result.rows_affected();
        if rows_affected > 0 {
            info!("系统初始化：将 {} 个进行中的游戏状态变更为暂停中", rows_affected);
        } else {
            info!("系统初始化：没有需要变更状态的游戏");
        }

        Ok(())
    }
}