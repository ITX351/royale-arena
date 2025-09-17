//! WebSocket服务实现

use axum::{
    extract::{
        ws::{Message, Utf8Bytes, WebSocket, WebSocketUpgrade},
        Path, Query, State,
    },
    response::Response,
};
use futures::{sink::SinkExt, stream::StreamExt};
use serde_json::json;

use crate::routes::AppState;
use super::models::*;
use crate::game::models::GameStatus;

use crate::websocket::models::GameState;
use crate::websocket::game_state_director_actions::*;
use crate::websocket::game_state_player_actions::*;

/// WebSocket服务
#[derive(Clone)]
pub struct WebSocketService {
    /// 应用状态
    app_state: AppState,
}

impl WebSocketService {
    /// 创建新的WebSocket服务
    pub fn new(app_state: AppState) -> Self {
        Self {
            app_state,
        }
    }

    /// 处理WebSocket连接升级
    pub async fn handle_websocket_upgrade(
        ws: WebSocketUpgrade,
        State(state): State<AppState>,
        Path(game_id): Path<String>,
        Query(query): Query<WebSocketAuthRequest>,
    ) -> Response {
        // 创建WebSocket服务实例
        let ws_service = WebSocketService::new(state);
        
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
                let success_msg = json!({
                    "type": "system_message",
                    "data": {
                        "message": "WebSocket connection established successfully"
                    }
                });
                
                if socket.send(Message::Text(Utf8Bytes::from(serde_json::to_string(&success_msg).unwrap()))).await.is_err() {
                    return;
                }

                // 根据用户类型处理连接
                match user_type {
                    ConnectionType::Player => {
                        self.handle_player_connection(&mut socket, &game_id, &auth_request.password).await;
                    }
                    ConnectionType::Director => {
                        self.handle_director_connection(&mut socket, &game_id, &auth_request.password).await;
                    }
                }
            }
            Err(error_msg) => {
                // 发送认证失败消息
                let error_response = json!({
                    "type": "error",
                    "data": {
                        "message": error_msg
                    }
                });
                
                let _ = socket.send(Message::Text(Utf8Bytes::from(serde_json::to_string(&error_response).unwrap()))).await;
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
        // 检查游戏是否存在且处于运行状态
        let game = self.app_state.game_service.get_game_by_id(game_id).await
            .map_err(|_| "Game not found".to_string())?;
        
        if game.status != GameStatus::Running {
            return Err("Game is not running".to_string());
        }

        // 根据用户类型验证密码
        match &auth_request.user_type {
            ConnectionType::Player => {
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
                
                Ok(ConnectionType::Player)
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
        &self,
        socket: &mut WebSocket,
        game_id: &str,
        player_password: &str,
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

        // 获取游戏状态
        let game = self.app_state.game_service.get_game_by_id(game_id).await.unwrap();
        let game_state_ref = self.app_state.game_state_manager.get_game_state(game_id, game.rules_config).await.unwrap();
        let game_state_guard = game_state_ref.read().await;

        // 构建玩家初始状态信息
        let player_state = if let Some(player) = game_state_guard.players.get(&actor.id) {
            json!({
                "location": player.location,
                "life": player.life,
                "strength": player.strength,
                "inventory": player.inventory,
                "equipped_item": player.equipped_item,
                "hand_item": player.hand_item,
                "votes": player.votes,
                "night_start_time": game_state_guard.night_start_time,
                "night_end_time": game_state_guard.night_end_time,
                "places": game_state_guard.places.keys().collect::<Vec<_>>(),
                "next_night_destroyed_places": game_state_guard.next_night_destroyed_places
            })
        } else {
            json!({
                "error": "Player not found in game state"
            })
        };

        // 发送玩家初始状态
        let init_msg = json!({
            "type": "player_update",
            "data": player_state
        });
        
        if socket.send(Message::Text(Utf8Bytes::from(serde_json::to_string(&init_msg).unwrap()))).await.is_err() {
            return;
        }

        // 处理玩家消息
        while let Some(Ok(msg)) = socket.next().await {
            if let Message::Text(text) = msg {
                match self.handle_player_message(game_id, &actor.id, &text).await {
                    Ok(response) => {
                        if socket.send(Message::Text(Utf8Bytes::from(response))).await.is_err() {
                            break;
                        }
                    }
                    Err(error_msg) => {
                        let error_response = json!({
                            "type": "error",
                            "data": {
                                "message": error_msg
                            }
                        });
                        if socket.send(Message::Text(Utf8Bytes::from(serde_json::to_string(&error_response).unwrap()))).await.is_err() {
                            break;
                        }
                    }
                }
            }
        }
    }

    /// 处理导演WebSocket连接
    async fn handle_director_connection(
        &self,
        socket: &mut WebSocket,
        game_id: &str,
        _director_password: &str,
    ) {
        // 获取游戏状态
        let game = self.app_state.game_service.get_game_by_id(game_id).await.unwrap();
        let game_state_ref = self.app_state.game_state_manager.get_game_state(game_id, game.rules_config).await.unwrap();
        let game_state_guard = game_state_ref.read().await;

        // 构建导演初始状态信息
        let director_state = json!({
            "game_status": "running", // 已经验证过游戏状态为running
            "night_start_time": game_state_guard.night_start_time,
            "night_end_time": game_state_guard.night_end_time,
            "players": game_state_guard.players.clone(),
            "places": game_state_guard.places.clone()
        });

        // 发送导演初始状态
        let init_msg = json!({
            "type": "game_state",
            "data": director_state
        });
        
        if socket.send(Message::Text(Utf8Bytes::from(serde_json::to_string(&init_msg).unwrap()))).await.is_err() {
            return;
        }

        // 处理导演消息
        while let Some(Ok(msg)) = socket.next().await {
            if let Message::Text(text) = msg {
                match self.handle_director_message(game_id, &text).await {
                    Ok(response) => {
                        if socket.send(Message::Text(Utf8Bytes::from(response))).await.is_err() {
                            break;
                        }
                    }
                    Err(error_msg) => {
                        let error_response = json!({
                            "type": "error",
                            "data": {
                                "message": error_msg
                            }
                        });
                        if socket.send(Message::Text(Utf8Bytes::from(serde_json::to_string(&error_response).unwrap()))).await.is_err() {
                            break;
                        }
                    }
                }
            }
        }
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
        let game = self.app_state.game_service.get_game_by_id(game_id).await
            .map_err(|e| format!("Failed to get game: {}", e))?;
        let game_state_ref = self.app_state.game_state_manager.get_game_state(game_id, game.rules_config).await
            .map_err(|e| format!("Failed to get game state: {}", e))?;

        // 根据行动类型处理
        let result = match action {
            "born" => {
                let place_name = action_data.get("place_name").and_then(|v| v.as_str())
                    .ok_or("Missing place_name field")?;
                let mut game_state = game_state_ref.write().await;
                game_state.handle_born_action(player_id, place_name)
            }
            "move" => {
                let target_place = action_data.get("target_place").and_then(|v| v.as_str())
                    .ok_or("Missing target_place field")?;
                let mut game_state = game_state_ref.write().await;
                game_state.handle_move_action(player_id, target_place)
            }
            "search" => {
                let mut game_state = game_state_ref.write().await;
                game_state.handle_search_action(player_id)
            }
            "pick" => {
                let mut game_state = game_state_ref.write().await;
                game_state.handle_pick_action(player_id)
            }
            "attack" => {
                let mut game_state = game_state_ref.write().await;
                game_state.handle_attack_action(player_id)
            }
            "equip" => {
                let item_id = action_data.get("item_id").and_then(|v| v.as_str())
                    .ok_or("Missing item_id field")?;
                let mut game_state = game_state_ref.write().await;
                game_state.handle_equip_action(player_id, item_id)
            }
            "use" => {
                let item_id = action_data.get("item_id").and_then(|v| v.as_str())
                    .ok_or("Missing item_id field")?;
                let mut game_state = game_state_ref.write().await;
                game_state.handle_use_action(player_id, item_id)
            }
            "throw" => {
                let item_id = action_data.get("item_id").and_then(|v| v.as_str())
                    .ok_or("Missing item_id field")?;
                let mut game_state = game_state_ref.write().await;
                game_state.handle_throw_action(player_id, item_id)
            }
            "deliver" => {
                let target_player_id = action_data.get("target_player_id").and_then(|v| v.as_str())
                    .ok_or("Missing target_player_id field")?;
                let message = action_data.get("message").and_then(|v| v.as_str())
                    .ok_or("Missing message field")?;
                let mut game_state = game_state_ref.write().await;
                game_state.handle_deliver_action(player_id, target_player_id, message)
            }
            "send" => {
                let message = action_data.get("message").and_then(|v| v.as_str())
                    .ok_or("Missing message field")?;
                let mut game_state = game_state_ref.write().await;
                game_state.handle_send_action(player_id, message)
            }
            _ => Err("Unknown action".to_string()),
        };

        // 序列化结果
        result.map(|v| v.to_string())
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
        let game = self.app_state.game_service.get_game_by_id(game_id).await
            .map_err(|e| format!("Failed to get game: {}", e))?;
        let game_state_ref = self.app_state.game_state_manager.get_game_state(game_id, game.rules_config).await
            .map_err(|e| format!("Failed to get game state: {}", e))?;

        // 根据行动类型处理
        let result = match action {
            "set_night_start_time" => {
                // 设置夜晚开始时间
                let timestamp = action_data.get("timestamp").and_then(|v| v.as_str())
                    .ok_or("Missing timestamp field")?;
                
                let mut game_state = game_state_ref.write().await;
                game_state.handle_set_night_start_time(timestamp)
                    .map(|v| v.to_string())
                    .map_err(|e| format!("Failed to set night start time: {}", e))
            }
            "set_night_end_time" => {
                // 设置夜晚结束时间
                let timestamp = action_data.get("timestamp").and_then(|v| v.as_str())
                    .ok_or("Missing timestamp field")?;
                
                let mut game_state = game_state_ref.write().await;
                game_state.handle_set_night_end_time(timestamp)
                    .map(|v| v.to_string())
                    .map_err(|e| format!("Failed to set night end time: {}", e))
            }
            "modify_place" => {
                // 调整地点状态
                let place_name = action_data.get("place_name").and_then(|v| v.as_str())
                    .ok_or("Missing place_name field")?;
                let is_destroyed = action_data.get("is_destroyed").and_then(|v| v.as_bool())
                    .ok_or("Missing is_destroyed field")?;
                
                let mut game_state = game_state_ref.write().await;
                game_state.handle_modify_place(place_name, is_destroyed)
                    .map(|v| v.to_string())
                    .map_err(|e| format!("Failed to modify place: {}", e))
            }
            "set_destroy_places" => {
                // 设置缩圈地点
                let places = action_data.get("places").and_then(|v| v.as_array())
                    .ok_or("Missing places field")?;
                
                let mut game_state = game_state_ref.write().await;
                game_state.handle_set_destroy_places(places)
                    .map(|v| v.to_string())
                    .map_err(|e| format!("Failed to set destroy places: {}", e))
            }
            "drop" => {
                // 空投
                let place_name = action_data.get("place_name").and_then(|v| v.as_str())
                    .ok_or("Missing place_name field")?;
                
                // 获取物品信息
                let item_data = action_data.get("item").ok_or("Missing item field")?;
                let item: crate::websocket::models::Item = serde_json::from_value(item_data.clone())
                    .map_err(|_| "Invalid item format".to_string())?;
                
                let mut game_state = game_state_ref.write().await;
                game_state.handle_drop(place_name, item)
                    .map(|v| v.to_string())
                    .map_err(|e| format!("Failed to drop item: {}", e))
            }
            "weather" => {
                // 调整天气
                let weather = action_data.get("weather").and_then(|v| v.as_f64())
                    .ok_or("Missing weather field")?;
                
                let mut game_state = game_state_ref.write().await;
                game_state.handle_weather(weather)
                    .map(|v| v.to_string())
                    .map_err(|e| format!("Failed to set weather: {}", e))
            }
            "life" => {
                // 调整生命值
                let player_id = action_data.get("player_id").and_then(|v| v.as_str())
                    .ok_or("Missing player_id field")?;
                let life_change = action_data.get("life_change").and_then(|v| v.as_i64())
                    .ok_or("Missing life_change field")?;
                
                let mut game_state = game_state_ref.write().await;
                game_state.handle_life(player_id, life_change)
                    .map(|v| v.to_string())
                    .map_err(|e| format!("Failed to adjust life: {}", e))
            }
            "strength" => {
                // 调整体力值
                let player_id = action_data.get("player_id").and_then(|v| v.as_str())
                    .ok_or("Missing player_id field")?;
                let strength_change = action_data.get("strength_change").and_then(|v| v.as_i64())
                    .ok_or("Missing strength_change field")?;
                
                let mut game_state = game_state_ref.write().await;
                game_state.handle_strength(player_id, strength_change)
                    .map(|v| v.to_string())
                    .map_err(|e| format!("Failed to adjust strength: {}", e))
            }
            "move_player" => {
                // 移动角色
                let player_id = action_data.get("player_id").and_then(|v| v.as_str())
                    .ok_or("Missing player_id field")?;
                let target_place = action_data.get("target_place").and_then(|v| v.as_str())
                    .ok_or("Missing target_place field")?;
                
                let mut game_state = game_state_ref.write().await;
                game_state.handle_move_player(player_id, target_place)
                    .map(|v| v.to_string())
                    .map_err(|e| format!("Failed to move player: {}", e))
            }
            "give" => {
                // 增减道具
                let target_type = action_data.get("target_type").and_then(|v| v.as_str())
                    .ok_or("Missing target_type field")?;
                let item_data = action_data.get("item").ok_or("Missing item field")?;
                let item: crate::websocket::models::Item = serde_json::from_value(item_data.clone())
                    .map_err(|_| "Invalid item format".to_string())?;
                
                let player_id = action_data.get("player_id").and_then(|v| v.as_str());
                let place_name = action_data.get("place_name").and_then(|v| v.as_str());
                
                let mut game_state = game_state_ref.write().await;
                game_state.handle_give(target_type, item, player_id.as_deref(), place_name.as_deref())
                    .map(|v| v.to_string())
                    .map_err(|e| format!("Failed to give item: {}", e))
            }
            "rope" | "unrope" => {
                // 捆绑/松绑
                let player_id = action_data.get("player_id").and_then(|v| v.as_str())
                    .ok_or("Missing player_id field")?;
                let action_type = action_data.get("action_type").and_then(|v| v.as_str())
                    .ok_or("Missing action_type field")?;
                
                let mut game_state = game_state_ref.write().await;
                game_state.handle_rope_action(player_id, action_type)
                    .map(|v| v.to_string())
                    .map_err(|e| format!("Failed to rope/unrope player: {}", e))
            }
            "broadcast" => {
                // 广播消息
                let message = action_data.get("message").and_then(|v| v.as_str())
                    .ok_or("Missing message field")?;
                
                let mut game_state = game_state_ref.write().await;
                game_state.handle_broadcast(message)
                    .map(|v| v.to_string())
                    .map_err(|e| format!("Failed to broadcast message: {}", e))
            }
            _ => Err("Unknown director action".to_string()),
        };

        result
    }

    /// 获取游戏状态（如果不存在则创建）
    async fn get_game_state(&self, game_id: &str) -> GameState {
        let game = self.app_state.game_service.get_game_by_id(game_id).await.unwrap();
        let game_state_ref = self.app_state.game_state_manager.get_game_state(game_id, game.rules_config).await.unwrap();
        let game_state_guard = game_state_ref.read().await;
        game_state_guard.clone()
    }

    /// 保存游戏状态到磁盘
    pub async fn save_game_state_to_disk(&self, game_id: &str) -> Result<(), String> {
        self.app_state.game_state_manager.save_game_state_to_disk(game_id).await
    }

    /// 从磁盘恢复游戏状态
    pub async fn load_game_state_from_disk(&self, game_id: &str) -> Result<(), String> {
        self.app_state.game_state_manager.load_game_state_from_disk(game_id).await
    }

    /// 开始游戏（等待中 → 进行中）
    pub async fn start_game(&self, game_id: &str) -> Result<(), String> {
        // 更新数据库中游戏状态为 "running"
        let result = sqlx::query!(
            "UPDATE games SET status = 'running', updated_at = CURRENT_TIMESTAMP WHERE id = ?",
            game_id
        )
        .execute(&self.app_state.director_service.pool)
        .await
        .map_err(|e| format!("Failed to update game status: {}", e))?;
        
        if result.rows_affected() == 0 {
            return Err("Game not found".to_string());
        }
        
        // 初始化游戏内存状态
        let game = self.app_state.game_service.get_game_by_id(game_id).await
            .map_err(|e| format!("Failed to get game: {}", e))?;
        self.app_state.game_state_manager.get_game_state(game_id, game.rules_config).await
            .map_err(|e| format!("Failed to initialize game state: {}", e))?;
        
        // 启动 WebSocket 监听器（已在连接处理中实现）
        
        Ok(())
    }

    /// 暂停游戏（进行中 → 暂停）
    pub async fn pause_game(&self, game_id: &str) -> Result<(), String> {
        // 更新数据库中游戏状态为 "paused"
        let result = sqlx::query!(
            "UPDATE games SET status = 'paused', updated_at = CURRENT_TIMESTAMP WHERE id = ?",
            game_id
        )
        .execute(&self.app_state.director_service.pool)
        .await
        .map_err(|e| format!("Failed to update game status: {}", e))?;
        
        if result.rows_affected() == 0 {
            return Err("Game not found".to_string());
        }
        
        // 将当前游戏状态序列化并保存到磁盘文件
        self.app_state.game_state_manager.save_game_state_to_disk(game_id).await
            .map_err(|e| format!("Failed to save game state to disk: {}", e))?;
        
        // 关闭 WebSocket 监听器（连接会自动断开）
        
        // 通知所有连接的客户端
        // 这里可以实现广播消息给所有连接的客户端
        
        Ok(())
    }

    /// 结束游戏（进行中 → 结束）
    pub async fn end_game(&self, game_id: &str) -> Result<(), String> {
        // 更新数据库中游戏状态为 "ended"
        let result = sqlx::query!(
            "UPDATE games SET status = 'ended', updated_at = CURRENT_TIMESTAMP WHERE id = ?",
            game_id
        )
        .execute(&self.app_state.director_service.pool)
        .await
        .map_err(|e| format!("Failed to update game status: {}", e))?;
        
        if result.rows_affected() == 0 {
            return Err("Game not found".to_string());
        }
        
        // 将当前游戏状态序列化并保存到磁盘文件
        self.save_game_state_to_disk(game_id).await
            .map_err(|e| format!("Failed to save game state to disk: {}", e))?;
        
        // 关闭 WebSocket 监听器（连接会自动断开）
        
        // 通知所有连接的客户端
        // 这里可以实现广播消息给所有连接的客户端
        
        Ok(())
    }

    /// 恢复游戏（暂停 → 进行中）
    pub async fn resume_game(&self, game_id: &str) -> Result<(), String> {
        // 更新数据库中游戏状态为 "running"
        let result = sqlx::query!(
            "UPDATE games SET status = 'running', updated_at = CURRENT_TIMESTAMP WHERE id = ?",
            game_id
        )
        .execute(&self.app_state.director_service.pool)
        .await
        .map_err(|e| format!("Failed to update game status: {}", e))?;
        
        if result.rows_affected() == 0 {
            return Err("Game not found".to_string());
        }
        
        // 从磁盘文件中恢复游戏状态
        self.app_state.game_state_manager.load_game_state_from_disk(game_id).await
            .map_err(|e| format!("Failed to load game state from disk: {}", e))?;
        
        // 启动 WebSocket 监听器（已在连接处理中实现）
        
        // 通知所有连接的客户端
        // 这里可以实现广播消息给所有连接的客户端
        
        Ok(())
    }
}
