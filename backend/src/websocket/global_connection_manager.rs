//! 全局WebSocket连接管理器
//! 负责管理所有游戏的GameConnectionManager实例

use dashmap::DashMap;
use std::sync::Arc;

use super::game_connection_manager::GameConnectionManager;

/// 全局WebSocket连接管理器
/// 管理所有游戏的GameConnectionManager实例
#[derive(Clone)]
pub struct GlobalConnectionManager {
    /// 游戏ID到连接管理器的映射
    managers: Arc<DashMap<String, Arc<GameConnectionManager>>>,
}

impl GlobalConnectionManager {
    /// 创建新的全局连接管理器
    pub fn new() -> Self {
        Self {
            managers: Arc::new(DashMap::new()),
        }
    }

    /// 获取指定游戏的连接管理器，如果不存在则创建新的
    pub fn get_manager(&self, game_id: String) -> Arc<GameConnectionManager> {
        // 尝试获取现有的GameConnectionManager
        if let Some(manager) = self.managers.get(&game_id) {
            return manager.clone();
        }

        // 如果不存在，则创建新的GameConnectionManager
        let new_manager = Arc::new(GameConnectionManager::new());
        self.managers.insert(game_id, new_manager.clone());
        new_manager
    }

    /// 断开并移除指定游戏的所有连接，返回被移除的连接管理器
    pub async fn remove_game_manager(&self, game_id: String) -> Option<Arc<GameConnectionManager>> {
        // 从映射中移除GameConnectionManager
        if let Some((_, manager)) = self.managers.remove(&game_id) {
            // 断开所有连接
            manager.disconnect_all_connections().await;
            Some(manager)
        } else {
            None
        }
    }

    // 获取所有游戏连接管理器
    // pub fn get_all_managers(&self) -> Vec<Arc<GameConnectionManager>> {
    //     self.managers
    //         .iter()
    //         .map(|entry| entry.value().clone())
    //         .collect()
    // }
}
