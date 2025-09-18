//! WebSocket游戏连接管理器
//! 负责管理单个游戏的所有WebSocket连接，包括玩家和导演连接

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde_json::{json, Value as JsonValue};
use crate::websocket::models::ConnectionType;

/// 连接句柄，用于标识每个WebSocket连接
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ConnectionHandle {
    /// 连接ID，用于唯一标识一个连接
    pub id: String,
    /// 用户ID（玩家ID或导演ID）
    pub user_id: String,
    /// 连接类型（玩家或导演）
    pub connection_type: ConnectionType,
}

/// WebSocket游戏连接管理器
/// 管理单个游戏的所有WebSocket连接
#[derive(Clone)]
pub struct GameConnectionManager {
    /// 玩家连接映射：玩家ID -> [连接句柄]
    player_connections: Arc<RwLock<HashMap<String, Vec<ConnectionHandle>>>>,
    /// 导演连接列表
    director_connections: Arc<RwLock<Vec<ConnectionHandle>>>,
    /// 实际的WebSocket连接：连接句柄 -> WebSocket发送端
    connections: Arc<RwLock<HashMap<ConnectionHandle, tokio::sync::mpsc::UnboundedSender<JsonValue>>>>,
}

impl GameConnectionManager {
    /// 创建新的游戏连接管理器
    pub fn new() -> Self {
        Self {
            player_connections: Arc::new(RwLock::new(HashMap::new())),
            director_connections: Arc::new(RwLock::new(Vec::new())),
            connections: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 添加新的WebSocket连接
    pub async fn add_connection(
        &self,
        user_id: String,
        connection_type: ConnectionType,
        sender: tokio::sync::mpsc::UnboundedSender<JsonValue>,
    ) -> ConnectionHandle {
        let connection_id = uuid::Uuid::new_v4().to_string();
        let handle = ConnectionHandle {
            id: connection_id.clone(),
            user_id: user_id.clone(),
            connection_type: connection_type.clone(),
        };

        // 存储连接
        self.connections.write().await.insert(handle.clone(), sender);

        // 根据连接类型添加到对应的映射中
        match connection_type {
            ConnectionType::Player => {
                let mut player_connections = self.player_connections.write().await;
                player_connections.entry(user_id).or_insert_with(Vec::new).push(handle.clone());
            }
            ConnectionType::Director => {
                self.director_connections.write().await.push(handle.clone());
            }
        }

        handle
    }

    /// 移除WebSocket连接
    pub async fn remove_connection(&self, handle: &ConnectionHandle) {
        // 从连接映射中移除
        self.connections.write().await.remove(handle);

        // 从对应的用户连接列表中移除
        match handle.connection_type {
            ConnectionType::Player => {
                let mut player_connections = self.player_connections.write().await;
                if let Some(connections) = player_connections.get_mut(&handle.user_id) {
                    connections.retain(|conn| conn.id != handle.id);
                    // 如果该玩家没有其他连接，移除该玩家的条目
                    if connections.is_empty() {
                        player_connections.remove(&handle.user_id);
                    }
                }
            }
            ConnectionType::Director => {
                self.director_connections.write().await.retain(|conn| conn.id != handle.id);
            }
        }
    }

    /// 获取指定玩家的所有连接句柄
    pub async fn get_player_connections(&self, player_id: &str) -> Vec<ConnectionHandle> {
        let player_connections = self.player_connections.read().await;
        player_connections.get(player_id).cloned().unwrap_or_else(Vec::new)
    }

    /// 获取所有导演连接句柄
    pub async fn get_director_connections(&self) -> Vec<ConnectionHandle> {
        self.director_connections.read().await.clone()
    }

    /// 获取所有连接句柄
    pub async fn get_all_connections(&self) -> Vec<ConnectionHandle> {
        let connections = self.connections.read().await;
        connections.keys().cloned().collect()
    }

    /// 向指定连接发送消息
    pub async fn send_message_to_connection(&self, handle: &ConnectionHandle, message: JsonValue) -> Result<(), String> {
        let connections = self.connections.read().await;
        if let Some(sender) = connections.get(handle) {
            sender.send(message).map_err(|e| format!("Failed to send message: {}", e))
        } else {
            Err("Connection not found".to_string())
        }
    }

    /// 向指定玩家的所有连接广播消息
    pub async fn broadcast_to_player(&self, player_id: &str, message: JsonValue) -> Result<(), String> {
        let handles = self.get_player_connections(player_id).await;
        for handle in handles {
            let _ = self.send_message_to_connection(&handle, message.clone()).await;
        }
        Ok(())
    }

    /// 向所有导演连接广播消息
    pub async fn broadcast_to_directors(&self, message: JsonValue) -> Result<(), String> {
        let handles = self.get_director_connections().await;
        for handle in handles {
            let _ = self.send_message_to_connection(&handle, message.clone()).await;
        }
        Ok(())
    }

    /// 向所有连接广播消息
    pub async fn broadcast_to_all(&self, message: JsonValue) -> Result<(), String> {
        let handles = self.get_all_connections().await;
        for handle in handles {
            let _ = self.send_message_to_connection(&handle, message.clone()).await;
        }
        Ok(())
    }

    /// 断开所有连接并向客户端发送断开消息
    pub async fn disconnect_all_connections(&self) {
        let disconnect_message = json!({
            "type": "system_message",
            "data": {
                "message": "Game has ended. Connection closed."
            }
        });

        // 向所有连接广播断开消息
        let _ = self.broadcast_to_all(disconnect_message).await;

        // 清理所有连接数据
        self.connections.write().await.clear();
        self.player_connections.write().await.clear();
        self.director_connections.write().await.clear();
    }
}