//! WebSocket服务实现

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, Query, State,
    },
    response::Response,
};
use futures::{sink::SinkExt, stream::StreamExt};
use serde_json::json;
use std::sync::Arc;

use tracing::debug;
use crate::routes::AppState;
use super::models::*;
use crate::game::models::GameStatus;

use crate::websocket::game_connection_manager::GameConnectionManager;
use crate::websocket::broadcaster::MessageBroadcaster;
use crate::websocket::player_action_scheduler::{PlayerActionScheduler, ActionParams};
use crate::websocket::director_action_scheduler::{DirectorActionScheduler, DirectorActionParams};

/// WebSocket服务
#[derive(Clone)]
pub struct WebSocketService {
    /// 应用状态
    app_state: AppState,

    /// 连接管理器
    connection_manager: Arc<GameConnectionManager>,
    /// 消息广播器
    message_broadcaster: MessageBroadcaster,
}

impl WebSocketService {
    /// 创建新的WebSocket服务
    pub fn new(app_state: AppState, connection_manager: Arc<GameConnectionManager>) -> Self {
        let message_broadcaster = MessageBroadcaster::new(connection_manager.as_ref().clone());
        
        Self {
            app_state,
            connection_manager,
            message_broadcaster,
        }
    }

    /// 处理WebSocket连接升级
    pub async fn handle_websocket_upgrade(
        ws: WebSocketUpgrade,
        State(state): State<AppState>,
        Path(game_id): Path<String>,
        Query(query): Query<WebSocketAuthRequest>,
    ) -> Response {
        // 获取游戏对应的连接管理器
        let game_connection_manager = state.global_connection_manager.get_manager(game_id.clone());
        // 创建WebSocket服务实例
        let ws_service = WebSocketService::new(state, game_connection_manager);
        // 升级WebSocket连接
        ws.on_upgrade(move |socket| ws_service.handle_websocket_connection(socket, game_id, query))
    }

    /// 处理WebSocket连接
    async fn handle_websocket_connection(
        self,
        mut socket: WebSocket,
        game_id: String,
        auth_request: WebSocketAuthRequest,
    ) {
        // 验证连接认证
        match self.authenticate_connection(&game_id, &auth_request).await {
            Ok(user_type) => {
                // 发送连接成功消息
                let websocket_message = super::message_formatter::system_message(json!({ "message": "WebSocket connection established successfully" }));
                if socket.send(websocket_message).await.is_err() {
                    return;
                }

                // 根据用户类型处理连接
                match user_type {
                    ConnectionType::Actor => {
                        self.handle_player_connection(socket, game_id, auth_request.password).await;
                    }
                    ConnectionType::Director => {
                        self.handle_director_connection(socket, game_id, auth_request.password).await;
                    }
                }
            }
            Err(error_msg) => {
                // 发送认证失败消息
                let websocket_message = super::message_formatter::error_message(json!({ "message": error_msg }));
                let _ = socket.send(websocket_message).await;
                // 关闭连接
                let _ = socket.close().await;
            }
        }
    }

    /// 验证WebSocket连接认证
    async fn authenticate_connection(
        &self,
        game_id: &str,
        auth_request: &WebSocketAuthRequest,
    ) -> Result<ConnectionType, String> {
        // 检查游戏是否存在
        let game = self.app_state.game_service.get_game_by_id(game_id).await
            .map_err(|_| "Game not found".to_string())?;
        
        // 检查游戏是否接受连接（只在游戏处于"进行时"或"暂停时"接受客户端连接）
        if !crate::game::global_game_state_manager::GlobalGameStateManager::is_status_accepting_connections(&game.status).await {
            // 检查游戏状态是否为等待中或已结束
            match game.status {
                GameStatus::Waiting => return Err("Game is waiting for start".to_string()),
                GameStatus::Ended => return Err("Game has ended".to_string()),
                _ => return Err("Game is not accepting connections".to_string()),
            }
        }

        // 根据用户类型验证密码
        match &auth_request.user_type {
            ConnectionType::Actor => {
                // 验证玩家密码
                let actor = sqlx::query!(
                    "SELECT id FROM actors WHERE game_id = ? AND password = ?",
                    game_id,
                    auth_request.password
                )
                .fetch_optional(&self.app_state.director_service.pool)
                .await
                .map_err(|_| "Database error".to_string())?;
                
                if actor.is_none() {
                    return Err("Invalid player password".to_string());
                }
                
                Ok(ConnectionType::Actor)
            }
            ConnectionType::Director => {
                // 验证导演密码
                let game_record = sqlx::query!(
                    "SELECT id FROM games WHERE id = ? AND director_password = ?",
                    game_id,
                    auth_request.password
                )
                .fetch_optional(&self.app_state.director_service.pool)
                .await
                .map_err(|_| "Database error".to_string())?;
                
                if game_record.is_none() {
                    return Err("Invalid director password".to_string());
                }
                
                Ok(ConnectionType::Director)
            }
        }
    }

    /// 处理玩家WebSocket连接
    async fn handle_player_connection(
        self,
        socket: WebSocket,
        game_id: String,
        player_password: String,
    ) {
        // 获取玩家信息
        let actor = sqlx::query!(
            "SELECT id, name FROM actors WHERE game_id = ? AND password = ?",
            game_id,
            player_password
        )
        .fetch_one(&self.app_state.director_service.pool)
        .await
        .unwrap(); // 已经验证过密码，这里不会失败

        let init_msg = {
            // let game = self.app_state.game_service.get_game_by_id(&game_id).await.unwrap();
            let game_state_ref = self.app_state.game_state_manager.get_game_state(&game_id).await.unwrap();
            let game_state_guard = game_state_ref.read().await;

            // 检查玩家是否在游戏状态中
            let player = game_state_guard.players.get(&actor.id).unwrap();

            // 生成玩家初始状态消息
            MessageBroadcaster::generate_player_message(&game_state_guard, player, None)
        };
        let websocket_message = super::message_formatter::game_state_message(init_msg);
        
        let (mut sender, mut receiver) = socket.split();
        if sender.send(websocket_message).await.is_err() {
            return;
        }

        // 创建消息通道用于连接管理
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<serde_json::Value>();
        
        // 添加连接到连接管理器
        let connection_handle = self.connection_manager.add_connection(
            actor.id.clone(),
            ConnectionType::Actor,
            tx
        ).await;

        // 处理来自连接管理器的消息
        let handle_messages = tokio::spawn(async move {
            while let Some(message) = rx.recv().await {
                // debug!("Player WS sending message: {:?}", &message);
                let websocket_message = super::message_formatter::game_state_message(message);
                if sender.send(websocket_message).await.is_err() {
                    break;
                }
            }
        });

        // 处理玩家消息
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Text(text) = msg {
                debug!("Player WS received message: {}", &text);
                if let Err(error_msg) = self.handle_player_message(&game_id, &actor.id, &text).await {
                    // 记录错误日志，方便后续排查调试
                    eprintln!("[WebSocket] Player message processing error: {}", error_msg);
                }
            }
        }

        // 连接断开时移除连接
        self.connection_manager.remove_connection(&connection_handle).await;
        handle_messages.abort();
    }

    /// 处理导演WebSocket连接
    async fn handle_director_connection(
        self,
        socket: WebSocket,
        game_id: String,
        _director_password: String,
    ) {
        let init_msg = {
            // let game = self.app_state.game_service.get_game_by_id(&game_id).await.unwrap();
            let game_state_ref = self.app_state.game_state_manager.get_game_state(&game_id).await.unwrap();
            let game_state_guard = game_state_ref.read().await;

            // 生成导演初始状态消息，action_result为空
            MessageBroadcaster::generate_director_message(&game_state_guard, None)
        };
        let websocket_message = super::message_formatter::game_state_message(init_msg);
        
        let (mut sender, mut receiver) = socket.split();
        if sender.send(websocket_message).await.is_err() {
            return;
        }

        // 创建消息通道用于连接管理
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<serde_json::Value>();
        
        // 添加连接到连接管理器
        let connection_handle = self.connection_manager.add_connection(
            "director".to_string(), // 导演使用固定ID
            ConnectionType::Director,
            tx
        ).await;

        // 处理来自连接管理器的消息
        let handle_messages = tokio::spawn(async move {
            while let Some(message) = rx.recv().await {
                // debug!("Director WS sending message: {:?}", &message);
                let websocket_message = super::message_formatter::game_state_message(message);
                if sender.send(websocket_message).await.is_err() {
                    break;
                }
            }
        });

        // 处理导演消息
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Text(text) = msg {
                debug!("Director WS received message: {}", &text);
                if let Err(error_msg) = self.handle_director_message(&game_id, &text).await {
                    // 记录错误日志，方便后续排查调试
                    eprintln!("[WebSocket] Director message processing error: {}", error_msg);
                }
            }
        }

        // 连接断开时移除连接
        self.connection_manager.remove_connection(&connection_handle).await;
        handle_messages.abort();
    }

    /// 处理玩家消息
    async fn handle_player_message(
        &self,
        game_id: &str,
        player_id: &str,
        message: &str,
    ) -> Result<String, String> {
        let client_message: WebSocketClientMessage = serde_json::from_str(message)
            .map_err(|_| "Invalid message format".to_string())?;

        match client_message.message_type {
            WebSocketMessageType::PlayerAction => {
                // 处理玩家行动
                self.process_player_action(game_id, player_id, client_message.data).await
            }
            _ => Err("Invalid message type for player".to_string()),
        }
    }

    /// 处理导演消息
    async fn handle_director_message(
        &self,
        game_id: &str,
        message: &str,
    ) -> Result<String, String> {
        let client_message: WebSocketClientMessage = serde_json::from_str(message)
            .map_err(|_| "Invalid message format".to_string())?;

        match client_message.message_type {
            WebSocketMessageType::DirectorAction => {
                // 处理导演控制
                self.process_director_action(game_id, client_message.data).await
            }
            _ => Err("Invalid message type for director".to_string()),
        }
    }

    /// 处理玩家行动
    async fn process_player_action(
        &self,
        game_id: &str,
        player_id: &str,
        action_data: serde_json::Value,
    ) -> Result<String, String> {
        // 获取行动类型
        let action = action_data.get("action").and_then(|v| v.as_str())
            .ok_or("Missing action field")?;
        
        // 获取游戏状态引用
        let game_state_ref = self.app_state.game_state_manager.get_game_state(&game_id).await
            .map_err(|e| format!("Failed to get game state: {}", e))?;

        let result = {
            // 获取可写的游戏状态锁
            let mut game_state = game_state_ref.write().await;

            // 使用调度器处理行动
            let action_params = ActionParams::from_json(&action_data)
                .map_err(|e| format!("Failed to parse action params: {}", e))?;
            
            PlayerActionScheduler::dispatch(
                &mut game_state,
                player_id,
                action,
                action_params
            )
        };

        // 根据动作结果进行广播
        match &result {
            Ok(action_result) => {
                // 获取更新后的游戏状态
                let updated_game_state = game_state_ref.read().await;
                
                // 使用新的广播器广播消息给相关玩家
                let _ = self.message_broadcaster.broadcast_to_players(&updated_game_state, &action_result.broadcast_players, action_result).await;
                
                // 根据broadcast_to_director字段判断是否向导演广播
                if action_result.broadcast_to_director {
                    let _ = self.message_broadcaster.broadcast_to_directors(&updated_game_state, action_result).await;
                }
                
                // 使用显式的消息类型而不是通过字符串内容判断
                let message_type = action_result.message_type.clone();
                
                // 仅在非Info类型消息时创建日志记录
                if message_type != crate::game::MessageType::Info {
                    // 为每个相关玩家创建日志记录
                    for broadcast_player_id in &action_result.broadcast_players {
                        let log_result = self.app_state.game_log_service.create_log(
                            game_id,
                            broadcast_player_id,
                            &action_result.log_message,
                            message_type.clone(),
                            action_result.timestamp,  // 传递ActionResult中的时间戳
                        ).await;
                        
                        // 忽略日志记录错误，但记录日志
                        if let Err(e) = log_result {
                            eprintln!("Failed to create log record: {}", e);
                        }
                    }
                }
            }
            Err(error_msg) => {
                // 记录错误日志，方便后续排查调试
                eprintln!("[WebSocket] Player action processing error: {}", error_msg);
            }
        }

        // 序列化结果
        result.map(|action_result| serde_json::to_string(&action_result.to_client_response()).unwrap_or_default())
    }

    /// 处理导演控制
    async fn process_director_action(
        &self,
        game_id: &str,
        action_data: serde_json::Value,
    ) -> Result<String, String> {
        // 获取行动类型
        let action = action_data.get("action").and_then(|v| v.as_str())
            .ok_or("Missing action field")?;
        // 获取游戏状态引用
        let game_state_ref = self.app_state.game_state_manager.get_game_state(&game_id).await
            .map_err(|e| format!("Failed to get game state: {}", e))?;

        let result = {
            let mut game_state = game_state_ref.write().await;
            
            // 使用调度器处理导演行动
            let action_params = DirectorActionParams::from_json(&action_data)
                .map_err(|e| format!("Failed to parse director action params: {}", e))?;
            
            DirectorActionScheduler::dispatch(
                &mut game_state,
                action,
                action_params
            )
        };

        // 根据动作结果进行广播
        match &result {
            Ok(action_result) => {
                // 获取更新后的游戏状态
                let updated_game_state = game_state_ref.read().await;
                
                // 使用新的广播器广播消息给相关玩家
                let _ = self.message_broadcaster.broadcast_to_players(&updated_game_state, &action_result.broadcast_players, action_result).await;
                
                // 根据broadcast_to_director字段判断是否向导演广播
                if action_result.broadcast_to_director {
                    let _ = self.message_broadcaster.broadcast_to_directors(&updated_game_state, action_result).await;
                }
                
                // 使用显式的消息类型而不是通过字符串内容判断
                let message_type = action_result.message_type.clone();
                
                // 仅在非Info类型消息时创建日志记录
                if message_type != crate::game::MessageType::Info {
                    // 为每个相关玩家创建日志记录
                    for broadcast_player_id in &action_result.broadcast_players {
                        let log_result = self.app_state.game_log_service.create_log(
                            game_id,
                            broadcast_player_id,
                            &action_result.log_message,
                            message_type.clone(),
                            action_result.timestamp,  // 传递ActionResult中的时间戳
                        ).await;
                        
                        // 忽略日志记录错误，但记录日志
                        if let Err(e) = log_result {
                            eprintln!("Failed to create log record: {}", e);
                        }
                    }
                }
            }
            Err(error_msg) => {
                // 记录错误日志，方便后续排查调试
                eprintln!("[WebSocket] Director action processing error: {}", error_msg);
            }
        }

        // 序列化结果
        result.map(|action_result| serde_json::to_string(&action_result.to_client_response()).unwrap_or_default())
    }
}