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
        let game_state = self.app_state.game_state_manager.get_game_state(game_id, game.rules_config).await.unwrap();

        // 构建玩家初始状态信息
        let player_state = if let Some(player) = game_state.players.get(&actor.id) {
            json!({
                "location": player.location,
                "life": player.life,
                "strength": player.strength,
                "inventory": player.inventory,
                "equipped_item": player.equipped_item,
                "hand_item": player.hand_item,
                "votes": player.votes,
                "night_start_time": game_state.night_start_time,
                "night_end_time": game_state.night_end_time,
                "places": game_state.places.keys().collect::<Vec<_>>(),
                "next_night_destroyed_places": game_state.next_night_destroyed_places
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
        let game_state = self.app_state.game_state_manager.get_game_state(game_id, game.rules_config).await.unwrap();

        // 构建导演初始状态信息
        let director_state = json!({
            "game_status": "running", // 已经验证过游戏状态为running
            "night_start_time": game_state.night_start_time,
            "night_end_time": game_state.night_end_time,
            "players": game_state.players,
            "places": game_state.places
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
        
        // 获取游戏状态
        let game = self.app_state.game_service.get_game_by_id(game_id).await
            .map_err(|e| format!("Failed to get game: {}", e))?;
        let mut game_state = self.app_state.game_state_manager.get_game_state(game_id, game.rules_config).await
            .map_err(|e| format!("Failed to get game state: {}", e))?;

        // 根据行动类型处理
        // 克隆player_id以避免借用问题
        let player_id_clone = player_id.to_string();
        let result = match action {
            "born" => {
                // 出生行动
                self.handle_born_action(&mut game_state, &player_id_clone, action_data).await
            }
            "move" => {
                // 移动行动
                self.handle_move_action(&mut game_state, &player_id_clone, action_data).await
            }
            "search" => {
                // 搜索行动
                self.handle_search_action(&mut game_state, &player_id_clone, action_data).await
            }
            "pick" => {
                // 捡拾行动
                self.handle_pick_action(&mut game_state, &player_id_clone, action_data).await
            }
            "attack" => {
                // 攻击行动
                self.handle_attack_action(&mut game_state, &player_id_clone, action_data).await
            }
            "equip" => {
                // 装备行动
                self.handle_equip_action(&mut game_state, &player_id_clone, action_data).await
            }
            "use" => {
                // 使用道具行动
                self.handle_use_action(&mut game_state, &player_id_clone, action_data).await
            }
            "throw" => {
                // 丢弃道具行动
                self.handle_throw_action(&mut game_state, &player_id_clone, action_data).await
            }
            "deliver" => {
                // 传音行动
                self.handle_deliver_action(&mut game_state, &player_id_clone, action_data).await
            }
            "send" => {
                // 对话导演行动
                self.handle_send_action(&mut game_state, &player_id_clone, action_data).await
            }
            _ => Err("Unknown action".to_string()),
        };

        // 更新游戏状态
        self.app_state.game_state_manager.update_game_state(game_id, game_state).await
            .map_err(|e| format!("Failed to update game state: {}", e))?;
        
        result
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
        
        // 获取游戏状态
        let game = self.app_state.game_service.get_game_by_id(game_id).await
            .map_err(|e| format!("Failed to get game: {}", e))?;
        let mut game_state = self.app_state.game_state_manager.get_game_state(game_id, game.rules_config).await
            .map_err(|e| format!("Failed to get game state: {}", e))?;

        // 根据行动类型处理
        let result = match action {
            "set_night_start_time" => {
                // 设置夜晚开始时间
                self.handle_set_night_start_time(&mut game_state, action_data).await
            }
            "set_night_end_time" => {
                // 设置夜晚结束时间
                self.handle_set_night_end_time(&mut game_state, action_data).await
            }
            "modify_place" => {
                // 调整地点状态
                self.handle_modify_place(&mut game_state, action_data).await
            }
            "set_destroy_places" => {
                // 设置缩圈地点
                self.handle_set_destroy_places(&mut game_state, action_data).await
            }
            "drop" => {
                // 空投
                self.handle_drop_action(&mut game_state, action_data).await
            }
            "weather" => {
                // 调整天气
                self.handle_weather_action(&mut game_state, action_data).await
            }
            "life" => {
                // 调整生命值
                self.handle_life_action(&mut game_state, action_data).await
            }
            "strength" => {
                // 调整体力值
                self.handle_strength_action(&mut game_state, action_data).await
            }
            "move_player" => {
                // 移动角色
                self.handle_move_player_action(&mut game_state, action_data).await
            }
            "give" => {
                // 增减道具
                self.handle_give_action(&mut game_state, action_data).await
            }
            "rope" | "unrope" => {
                // 捆绑/松绑
                self.handle_rope_action(&mut game_state, action_data).await
            }
            "broadcast" => {
                // 广播消息
                self.handle_broadcast_action(&mut game_state, action_data).await
            }
            _ => Err("Unknown director action".to_string()),
        };

        // 更新游戏状态
        self.app_state.game_state_manager.update_game_state(game_id, game_state).await
            .map_err(|e| format!("Failed to update game state: {}", e))?;
        
        result
    }

    /// 获取游戏状态（如果不存在则创建）
    async fn get_game_state(&self, game_id: &str) -> GameState {
        let game = self.app_state.game_service.get_game_by_id(game_id).await.unwrap();
        self.app_state.game_state_manager.get_game_state(game_id, game.rules_config).await.unwrap()
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

    // 以下方法是具体的行动处理方法，保持不变
    async fn handle_set_night_start_time(&self, game_state: &mut GameState, action_data: serde_json::Value) -> Result<String, String> {
        // 获取时间参数
        let timestamp = action_data.get("timestamp").and_then(|v| v.as_str())
            .ok_or("Missing timestamp field")?;
        
        // 解析时间
        let time = chrono::DateTime::parse_from_rfc3339(timestamp)
            .map_err(|_| "Invalid timestamp format")?
            .with_timezone(&chrono::Utc);
        
        // 更新游戏状态中的夜晚开始时间
        game_state.night_start_time = Some(time);
        
        // 向所有客户端广播更新后的时间设置
        let response = json!({
            "type": "game_state",
            "data": {
                "night_start_time": time
            }
        });
        
        Ok(serde_json::to_string(&response).map_err(|e| format!("Failed to serialize response: {}", e))?)
    }

    async fn handle_set_night_end_time(&self, game_state: &mut GameState, action_data: serde_json::Value) -> Result<String, String> {
        // 获取时间参数
        let timestamp = action_data.get("timestamp").and_then(|v| v.as_str())
            .ok_or("Missing timestamp field")?;
        
        // 解析时间
        let time = chrono::DateTime::parse_from_rfc3339(timestamp)
            .map_err(|_| "Invalid timestamp format")?
            .with_timezone(&chrono::Utc);
        
        // 更新游戏状态中的夜晚结束时间
        game_state.night_end_time = Some(time);
        
        // 向所有客户端广播更新后的时间设置
        let response = json!({
            "type": "game_state",
            "data": {
                "night_end_time": time
            }
        });
        
        Ok(serde_json::to_string(&response).map_err(|e| format!("Failed to serialize response: {}", e))?)
    }

    async fn handle_modify_place(&self, game_state: &mut GameState, action_data: serde_json::Value) -> Result<String, String> {
        // 获取地点名称和状态参数
        let place_name = action_data.get("place_name").and_then(|v| v.as_str())
            .ok_or("Missing place_name field")?;
        let is_destroyed = action_data.get("is_destroyed").and_then(|v| v.as_bool())
            .ok_or("Missing is_destroyed field")?;
        
        // 更新指定地点的摧毁状态
        if let Some(place) = game_state.places.get_mut(place_name) {
            place.is_destroyed = is_destroyed;
            
            // 检查地点内的玩家是否受影响
            // 如果地点被摧毁，需要处理在该地点的玩家
            
            // 向相关客户端广播地点状态更新
            let response = json!({
                "type": "game_state",
                "data": {
                    "place": {
                        "name": place_name,
                        "is_destroyed": is_destroyed
                    }
                }
            });
            
            Ok(serde_json::to_string(&response).map_err(|e| format!("Failed to serialize response: {}", e))?)
        } else {
            Err("Place not found".to_string())
        }
    }

    async fn handle_set_destroy_places(&self, game_state: &mut GameState, action_data: serde_json::Value) -> Result<String, String> {
        // 获取缩圈地点列表
        let places = action_data.get("places").and_then(|v| v.as_array())
            .ok_or("Missing places field")?;
        
        // 更新下一夜晚缩圈地点集合
        game_state.next_night_destroyed_places = places.iter()
            .filter_map(|v| v.as_str())
            .map(|s| s.to_string())
            .collect();
        
        // 向所有客户端广播更新后的缩圈地点集合
        let response = json!({
            "type": "game_state",
            "data": {
                "next_night_destroyed_places": game_state.next_night_destroyed_places
            }
        });
        
        Ok(serde_json::to_string(&response).map_err(|e| format!("Failed to serialize response: {}", e))?)
    }

    async fn handle_drop_action(&self, game_state: &mut GameState, action_data: serde_json::Value) -> Result<String, String> {
        // 获取地点名称和物品信息
        let place_name = action_data.get("place_name").and_then(|v| v.as_str())
            .ok_or("Missing place_name field")?;
        
        // 获取物品信息
        let item_data = action_data.get("item").ok_or("Missing item field")?;
        let item: Item = serde_json::from_value(item_data.clone())
            .map_err(|_| "Invalid item format")?;
        
        // 在指定地点添加空投物品
        if let Some(place) = game_state.places.get_mut(place_name) {
            place.items.push(item);
            
            // 向所有客户端广播地点物品更新
            let response = json!({
                "type": "game_state",
                "data": {
                    "place": {
                        "name": place_name,
                        "items": place.items
                    }
                }
            });
            
            Ok(serde_json::to_string(&response).map_err(|e| format!("Failed to serialize response: {}", e))?)
        } else {
            Err("Place not found".to_string())
        }
    }

    async fn handle_weather_action(&self, game_state: &mut GameState, action_data: serde_json::Value) -> Result<String, String> {
        // 获取天气条件值
        let weather = action_data.get("weather").and_then(|v| v.as_f64())
            .ok_or("Missing weather field")?;
        
        // 更新天气条件值
        game_state.weather = weather;
        
        // 向所有客户端广播天气更新
        let response = json!({
            "type": "game_state",
            "data": {
                "weather": weather
            }
        });
        
        Ok(serde_json::to_string(&response).map_err(|e| format!("Failed to serialize response: {}", e))?)
    }

    async fn handle_life_action(&self, game_state: &mut GameState, action_data: serde_json::Value) -> Result<String, String> {
        // 获取玩家ID和生命值变化
        let player_id = action_data.get("player_id").and_then(|v| v.as_str())
            .ok_or("Missing player_id field")?;
        let life_change = action_data.get("life_change").and_then(|v| v.as_i64())
            .ok_or("Missing life_change field")?;
        
        // 更新指定玩家生命值
        if let Some(player) = game_state.players.get_mut(player_id) {
            player.life += life_change as i32;
            
            // 检查玩家是否死亡或复活
            if player.life <= 0 {
                player.life = 0;
                player.is_alive = false;
            } else if player.life > 0 && !player.is_alive {
                player.is_alive = true;
            }
            
            // 向相关客户端广播玩家状态更新
            let response = json!({
                "type": "player_update",
                "data": {
                    "player_id": player_id,
                    "life": player.life,
                    "is_alive": player.is_alive
                }
            });
            
            Ok(serde_json::to_string(&response).map_err(|e| format!("Failed to serialize response: {}", e))?)
        } else {
            Err("Player not found".to_string())
        }
    }

    async fn handle_strength_action(&self, game_state: &mut GameState, action_data: serde_json::Value) -> Result<String, String> {
        // 获取玩家ID和体力值变化
        let player_id = action_data.get("player_id").and_then(|v| v.as_str())
            .ok_or("Missing player_id field")?;
        let strength_change = action_data.get("strength_change").and_then(|v| v.as_i64())
            .ok_or("Missing strength_change field")?;
        
        // 更新指定玩家体力值
        if let Some(player) = game_state.players.get_mut(player_id) {
            player.strength += strength_change as i32;
            
            // 确保体力值在合理范围内
            if player.strength < 0 {
                player.strength = 0;
            }
            
            // 向所有客户端广播玩家状态更新
            let response = json!({
                "type": "player_update",
                "data": {
                    "player_id": player_id,
                    "strength": player.strength
                }
            });
            
            Ok(serde_json::to_string(&response).map_err(|e| format!("Failed to serialize response: {}", e))?)
        } else {
            Err("Player not found".to_string())
        }
    }

    async fn handle_move_player_action(&self, game_state: &mut GameState, action_data: serde_json::Value) -> Result<String, String> {
        // 获取玩家ID和目标地点
        let player_id = action_data.get("player_id").and_then(|v| v.as_str())
            .ok_or("Missing player_id field")?;
        let target_place = action_data.get("target_place").and_then(|v| v.as_str())
            .ok_or("Missing target_place field")?;
        
        // 验证目标地点是否存在且未被摧毁
        if let Some(place) = game_state.places.get(target_place) {
            if place.is_destroyed {
                return Err("Target place is destroyed".to_string());
            }
        } else {
            return Err("Target place not found".to_string());
        }
        
        // 更新玩家位置
        if let Some(player) = game_state.players.get_mut(player_id) {
            // 从当前地点移除玩家
            if let Some(current_place) = game_state.places.get_mut(&player.location) {
                current_place.players.retain(|id| id != player_id);
            }
            
            // 更新玩家位置到目标地点
            player.location = target_place.to_string();
            
            // 将玩家添加到目标地点的玩家列表中
            if let Some(target_place_obj) = game_state.places.get_mut(target_place) {
                target_place_obj.players.push(player_id.to_string());
            }
            
            // 向所有客户端广播玩家位置更新
            let response = json!({
                "type": "player_update",
                "data": {
                    "player_id": player_id,
                    "location": target_place
                }
            });
            
            Ok(serde_json::to_string(&response).map_err(|e| format!("Failed to serialize response: {}", e))?)
        } else {
            Err("Player not found".to_string())
        }
    }

    async fn handle_give_action(&self, game_state: &mut GameState, action_data: serde_json::Value) -> Result<String, String> {
        // 获取目标（玩家或地点）
        let target_type = action_data.get("target_type").and_then(|v| v.as_str())
            .ok_or("Missing target_type field")?;
        let item_data = action_data.get("item").ok_or("Missing item field")?;
        let item: Item = serde_json::from_value(item_data.clone())
            .map_err(|_| "Invalid item format")?;
        
        match target_type {
            "player" => {
                // 给玩家道具
                let player_id = action_data.get("player_id").and_then(|v| v.as_str())
                    .ok_or("Missing player_id field")?;
                
                if let Some(player) = game_state.players.get_mut(player_id) {
                    // 将物品添加到指定玩家背包
                    player.inventory.push(item);
                    
                    // 向相关客户端广播物品更新
                    let response = json!({
                        "type": "player_update",
                        "data": {
                            "player_id": player_id,
                            "inventory": player.inventory
                        }
                    });
                    
                    Ok(serde_json::to_string(&response).map_err(|e| format!("Failed to serialize response: {}", e))?)
                } else {
                    Err("Player not found".to_string())
                }
            }
            "place" => {
                // 在地点放置道具
                let place_name = action_data.get("place_name").and_then(|v| v.as_str())
                    .ok_or("Missing place_name field")?;
                
                if let Some(place) = game_state.places.get_mut(place_name) {
                    // 将物品添加到指定地点物品列表
                    place.items.push(item);
                    
                    // 向相关客户端广播物品更新
                    let response = json!({
                        "type": "game_state",
                        "data": {
                            "place": {
                                "name": place_name,
                                "items": place.items
                            }
                        }
                    });
                    
                    Ok(serde_json::to_string(&response).map_err(|e| format!("Failed to serialize response: {}", e))?)
                } else {
                    Err("Place not found".to_string())
                }
            }
            _ => Err("Invalid target type".to_string())
        }
    }

    async fn handle_rope_action(&self, game_state: &mut GameState, action_data: serde_json::Value) -> Result<String, String> {
        // 获取玩家ID和操作类型
        let player_id = action_data.get("player_id").and_then(|v| v.as_str())
            .ok_or("Missing player_id field")?;
        let action_type = action_data.get("action_type").and_then(|v| v.as_str())
            .ok_or("Missing action_type field")?;
        
        // 更新指定玩家的绑定状态
        if let Some(player) = game_state.players.get_mut(player_id) {
            match action_type {
                "rope" => {
                    player.is_bound = true;
                }
                "unrope" => {
                    player.is_bound = false;
                }
                _ => return Err("Invalid action type".to_string())
            }
            
            // 向该玩家发送状态更新
            let response = json!({
                "type": "player_update",
                "data": {
                    "player_id": player_id,
                    "is_bound": player.is_bound
                }
            });
            
            Ok(serde_json::to_string(&response).map_err(|e| format!("Failed to serialize response: {}", e))?)
        } else {
            Err("Player not found".to_string())
        }
    }

    async fn handle_broadcast_action(&self, _game_state: &mut GameState, action_data: serde_json::Value) -> Result<String, String> {
        // 获取广播消息
        let message = action_data.get("message").and_then(|v| v.as_str())
            .ok_or("Missing message field")?;
        
        // 向所有客户端发送系统消息
        let response = json!({
            "type": "system_message",
            "data": {
                "message": message
            }
        });
        
        Ok(serde_json::to_string(&response).map_err(|e| format!("Failed to serialize response: {}", e))?)
    }

    // 玩家行为处理方法实现
    async fn handle_born_action(&self, game_state: &mut GameState, player_id: &str, action_data: serde_json::Value) -> Result<String, String> {
        // 获取玩家引用
        let player = game_state.players.get_mut(player_id).ok_or("Player not found".to_string())?;
        
        // 检查玩家是否已经执行过出生
        // 这里我们假设玩家出生后位置不为空字符串
        if !player.location.is_empty() {
            return Err("Player has already been born".to_string());
        }
        
        // 获取指定地点
        let place_name = action_data.get("place_name").and_then(|v| v.as_str())
            .ok_or("Missing place_name field")?;
        
        // 验证指定地点是否存在且未被摧毁
        if let Some(place) = game_state.places.get(place_name) {
            if place.is_destroyed {
                return Err("Place is destroyed".to_string());
            }
            
            // 更新玩家位置到指定地点
            player.location = place_name.to_string();
            
            // 将玩家添加到地点的玩家列表中
            let place_mut = game_state.places.get_mut(place_name).unwrap();
            place_mut.players.push(player.id.clone());
            
            // 向该玩家发送位置更新结果
            let response = json!({
                "type": "player_update",
                "data": {
                    "location": place_name
                }
            });
            
            Ok(serde_json::to_string(&response).map_err(|e| format!("Failed to serialize response: {}", e))?)
        } else {
            Err("Place not found".to_string())
        }
    }

    async fn handle_move_action(&self, game_state: &mut GameState, player_id: &str, action_data: serde_json::Value) -> Result<String, String> {
        // 获取玩家引用
        let player = game_state.players.get_mut(player_id).ok_or("Player not found".to_string())?;
        
        // 检查前置条件：玩家处于存活状态，有足够体力
        if !player.is_alive {
            return Err("Player is not alive".to_string());
        }
        
        // 获取目标地点
        let target_place = action_data.get("target_place").and_then(|v| v.as_str())
            .ok_or("Missing target_place field")?;
        
        // 验证目标地点是否存在且未被摧毁
        if let Some(place) = game_state.places.get(target_place) {
            if place.is_destroyed {
                return Err("Target place is destroyed".to_string());
            }
            
            // 消耗体力值（根据规则配置）
            // 这里我们简化处理，假设每次移动消耗5点体力
            let move_cost = 5;
            if player.strength < move_cost {
                return Err("Not enough strength".to_string());
            }
            player.strength -= move_cost;
            
            // 从当前地点移除玩家
            if let Some(current_place) = game_state.places.get_mut(&player.location) {
                current_place.players.retain(|id| id != &player.id);
            }
            
            // 更新玩家位置到目标地点
            player.location = target_place.to_string();
            
            // 将玩家添加到目标地点的玩家列表中
            if let Some(target_place_obj) = game_state.places.get_mut(target_place) {
                target_place_obj.players.push(player.id.clone());
            }
            
            // 向该玩家发送位置更新结果
            let response = json!({
                "type": "player_update",
                "data": {
                    "location": target_place,
                    "strength": player.strength
                }
            });
            
            Ok(serde_json::to_string(&response).map_err(|e| format!("Failed to serialize response: {}", e))?)
        } else {
            Err("Target place not found".to_string())
        }
    }

    async fn handle_search_action(&self, game_state: &mut GameState, player_id: &str, _action_data: serde_json::Value) -> Result<String, String> {
        // 获取玩家状态信息（避免借用冲突）
        let (player_location, player_alive, player_strength, player_last_search_time) = {
            let player = game_state.players.get(player_id).ok_or("Player not found".to_string())?;
            (
                player.location.clone(),
                player.is_alive,
                player.strength,
                player.last_search_time,
            )
        };
        
        // 检查前置条件：玩家处于存活状态，有足够体力，未处于搜索冷却期
        if !player_alive {
            return Err("Player is not alive".to_string());
        }
        
        // 检查体力
        let search_cost = 5; // 假设搜索消耗5点体力
        if player_strength < search_cost {
            return Err("Not enough strength".to_string());
        }
        
        // 检查搜索冷却期（简化处理，假设冷却期为30秒）
        let search_cooldown = 30;
        if let Some(last_search_time) = player_last_search_time {
            let elapsed = chrono::Utc::now().signed_duration_since(last_search_time);
            if elapsed.num_seconds() < search_cooldown {
                return Err("Search is in cooldown period".to_string());
            }
        }
        
        // 更新玩家状态
        {
            let player = game_state.players.get_mut(player_id).ok_or("Player not found".to_string())?;
            player.strength -= search_cost;
            player.last_search_time = Some(chrono::Utc::now());
        }
        
        // 随机确定搜索结果（玩家/物品/空）
        // 这里我们简化处理，随机生成结果
        use rand::Rng;
        let mut rng = rand::rng();
        let result_type: u32 = rng.random_range(0..3);
        
        match result_type {
            0 => {
                // 搜索到玩家
                // 查找当前地点的其他玩家（先获取地点信息，避免借用冲突）
                let other_player_ids = {
                    // 先检查地点是否存在
                    if game_state.places.contains_key(&player_location) {
                        // 获取地点玩家列表的副本
                        let place = &game_state.places[&player_location];
                        let mut place_players = Vec::new();
                        for id in &place.players {
                            place_players.push(id.clone());
                        }
                        
                        // 过滤掉当前玩家
                        let mut filtered_ids = Vec::new();
                        for id in place_players {
                            if id != player_id {
                                filtered_ids.push(id.clone());
                            }
                        }
                        filtered_ids
                    } else {
                        Vec::new()
                    }
                };
                
                if !other_player_ids.is_empty() {
                    // 随机选择一个玩家
                    let target_player_id = &other_player_ids[rng.random_range(0..other_player_ids.len())];
                    
                    // 获取目标玩家信息
                    let (target_player_id_clone, target_player_name) = {
                        if let Some(target_player) = game_state.players.get(target_player_id) {
                            (target_player.id.clone(), target_player.name.clone())
                        } else {
                            return Err("Target player not found".to_string());
                        }
                    };
                    
                    // 根据天气条件确定结果可见性
                    let is_visible = rng.random_bool(game_state.weather);
                    
                    // 更新玩家的上次搜索结果
                    {
                        let player = game_state.players.get_mut(player_id).ok_or("Player not found".to_string())?;
                        player.last_search_result = Some(SearchResult {
                            target_type: SearchResultType::Player,
                            target_id: target_player_id_clone.clone(),
                            target_name: target_player_name.clone(),
                            is_visible,
                        });
                    }
                    
                    // 获取更新后的玩家状态
                    let (player_strength, player_last_search_time) = {
                        let player = game_state.players.get(player_id).ok_or("Player not found".to_string())?;
                        (player.strength, player.last_search_time)
                    };
                    
                    // 向该玩家发送搜索结果
                    let response = json!({
                        "type": "player_update",
                        "data": {
                            "last_search_result": {
                                "target_type": "player",
                                "target_id": target_player_id_clone,
                                "target_name": target_player_name,
                                "is_visible": is_visible
                            },
                            "strength": player_strength,
                            "last_search_time": player_last_search_time
                        }
                    });
                    
                    return Ok(serde_json::to_string(&response).map_err(|e| format!("Failed to serialize response: {}", e))?);
                }
                
                // 如果没有其他玩家，返回空结果
                {
                    let player = game_state.players.get_mut(player_id).ok_or("Player not found".to_string())?;
                    player.last_search_result = None;
                }
                
                // 获取更新后的玩家状态
                let (player_strength, player_last_search_time) = {
                    let player = game_state.players.get(player_id).ok_or("Player not found".to_string())?;
                    (player.strength, player.last_search_time)
                };
                
                let response = json!({
                    "type": "player_update",
                    "data": {
                        "last_search_result": null,
                        "strength": player_strength,
                        "last_search_time": player_last_search_time
                    }
                });
                Ok(serde_json::to_string(&response).map_err(|e| format!("Failed to serialize response: {}", e))?)
            }
            1 => {
                // 搜索到物品
                // 先获取地点信息，避免借用冲突
                let place_info = {
                    if let Some(place) = game_state.places.get(&player_location) {
                        if !place.items.is_empty() {
                            // 随机选择一个物品
                            let item_index = rng.random_range(0..place.items.len());
                            let item = &place.items[item_index];
                            Some((item.id.clone(), item.name.clone()))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                };
                
                if let Some((item_id, item_name)) = place_info {
                    // 根据天气条件确定结果可见性
                    let is_visible = rng.random_bool(game_state.weather);
                    
                    // 更新玩家的上次搜索结果
                    {
                        let player = game_state.players.get_mut(player_id).ok_or("Player not found".to_string())?;
                        player.last_search_result = Some(SearchResult {
                            target_type: SearchResultType::Item,
                            target_id: item_id.clone(),
                            target_name: item_name.clone(),
                            is_visible,
                        });
                    }
                    
                    // 获取更新后的玩家状态
                    let (player_strength, player_last_search_time) = {
                        let player = game_state.players.get(player_id).ok_or("Player not found".to_string())?;
                        (player.strength, player.last_search_time)
                    };
                    
                    // 向该玩家发送搜索结果
                    let response = json!({
                        "type": "player_update",
                        "data": {
                            "last_search_result": {
                                "target_type": "item",
                                "target_id": item_id,
                                "target_name": item_name,
                                "is_visible": is_visible
                            },
                            "strength": player_strength,
                            "last_search_time": player_last_search_time
                        }
                    });
                    
                    Ok(serde_json::to_string(&response).map_err(|e| format!("Failed to serialize response: {}", e))?)
                } else {
                    // 地点没有物品，返回空结果
                    {
                        let player = game_state.players.get_mut(player_id).ok_or("Player not found".to_string())?;
                        player.last_search_result = None;
                    }
                    
                    // 获取更新后的玩家状态
                    let (player_strength, player_last_search_time) = {
                        let player = game_state.players.get(player_id).ok_or("Player not found".to_string())?;
                        (player.strength, player.last_search_time)
                    };
                    
                    let response = json!({
                        "type": "player_update",
                        "data": {
                            "last_search_result": null,
                            "strength": player_strength,
                            "last_search_time": player_last_search_time
                        }
                    });
                    Ok(serde_json::to_string(&response).map_err(|e| format!("Failed to serialize response: {}", e))?)
                }
            }
            _ => {
                // 搜索结果为空
                {
                    let player = game_state.players.get_mut(player_id).ok_or("Player not found".to_string())?;
                    player.last_search_result = None;
                }
                
                // 获取更新后的玩家状态
                let (player_strength, player_last_search_time) = {
                    let player = game_state.players.get(player_id).ok_or("Player not found".to_string())?;
                    (player.strength, player.last_search_time)
                };
                
                let response = json!({
                    "type": "player_update",
                    "data": {
                        "last_search_result": null,
                        "strength": player_strength,
                        "last_search_time": player_last_search_time
                    }
                });
                Ok(serde_json::to_string(&response).map_err(|e| format!("Failed to serialize response: {}", e))?)
            }
        }
    }

    async fn handle_pick_action(&self, game_state: &mut GameState, player_id: &str, _action_data: serde_json::Value) -> Result<String, String> {
        // 检查玩家是否存在且处于存活状态，上一次搜索结果为物品
        {
            let player = game_state.players.get(player_id).ok_or("Player not found".to_string())?;
            
            if !player.is_alive {
                return Err("Player is not alive".to_string());
            }
            
            let last_search_result_valid = if let Some(ref search_result) = player.last_search_result {
                search_result.target_type == SearchResultType::Item
            } else {
                false
            };
            
            if !last_search_result_valid {
                return Err("Last search result is not an item".to_string());
            }
        }
        
        // 获取搜索结果信息和玩家位置
        let (player_last_search_result, player_location) = {
            let player = game_state.players.get(player_id).ok_or("Player not found".to_string())?;
            (player.last_search_result.clone(), player.location.clone())
        };
        
        let item_id = if let Some(ref search_result) = player_last_search_result {
            search_result.target_id.clone()
        } else {
            return Err("No previous search result".to_string());
        };
        
        // 验证上一次搜索到的物品是否仍然存在
        if let Some(place) = game_state.places.get_mut(&player_location) {
            let item_index = place.items.iter().position(|item| item.id == item_id);
            if let Some(item_index) = item_index {
                // 从地点物品列表中移除物品
                let item = place.items.remove(item_index);
                
                // 将物品添加到玩家背包
                {
                    let player = game_state.players.get_mut(player_id).ok_or("Player not found".to_string())?;
                    player.inventory.push(item);
                }
                
                // 获取更新后的玩家背包
                let player_inventory = {
                    let player = game_state.players.get(player_id).ok_or("Player not found".to_string())?;
                    player.inventory.clone()
                };
                
                // 向该玩家发送背包更新
                let response = json!({
                    "type": "player_update",
                    "data": {
                        "inventory": player_inventory
                    }
                });
                
                Ok(serde_json::to_string(&response).map_err(|e| format!("Failed to serialize response: {}", e))?)
            } else {
                // 物品不存在，向该玩家发送捡拾失败消息
                let response = json!({
                    "type": "error",
                    "data": {
                        "message": "Item no longer exists"
                    }
                });
                Ok(serde_json::to_string(&response).map_err(|e| format!("Failed to serialize response: {}", e))?)
            }
        } else {
            Err("Player location not found".to_string())
        }
    }

    async fn handle_attack_action(&self, game_state: &mut GameState, player_id: &str, _action_data: serde_json::Value) -> Result<String, String> {
        // 检查前置条件：玩家处于存活状态，上一次搜索结果为玩家，玩家装备了武器
        {
            let player = game_state.players.get(player_id).ok_or("Player not found".to_string())?;
            
            if !player.is_alive {
                return Err("Player is not alive".to_string());
            }
            
            if player.equipped_item.is_none() {
                return Err("Player has no equipped item".to_string());
            }
            
            if player.last_search_result.is_none() {
                return Err("No previous search result".to_string());
            }
            
            let last_search_result_valid = if let Some(ref search_result) = player.last_search_result {
                search_result.target_type == SearchResultType::Player
            } else {
                false
            };
            
            if !last_search_result_valid {
                return Err("Last search result is not a player".to_string());
            }
        }
        
        // 获取搜索结果信息和玩家位置
        let (player_last_search_result, player_location) = {
            let player = game_state.players.get(player_id).ok_or("Player not found".to_string())?;
            (player.last_search_result.clone(), player.location.clone())
        };
        
        // 获取搜索结果信息
        let target_player_id = if let Some(ref search_result) = player_last_search_result {
            search_result.target_id.clone()
        } else {
            return Err("No previous search result".to_string());
        };
        
        // 验证目标玩家是否存在且在同一地点
        let (target_player_location, target_player_alive) = {
            if let Some(target_player) = game_state.players.get(&target_player_id) {
                (target_player.location.clone(), target_player.is_alive)
            } else {
                return Err("Target player not found".to_string());
            }
        };
        
        // 验证目标玩家是否在同一地点
        if target_player_location != player_location {
            // 目标玩家已离开
            let response = json!({
                "type": "error",
                "data": {
                    "message": "Target player has left the location"
                }
            });
            return Ok(serde_json::to_string(&response).map_err(|e| format!("Failed to serialize response: {}", e))?);
        }
        
        // 检查目标玩家是否已死亡
        if !target_player_alive {
            // 目标玩家已死亡
            let response = json!({
                "type": "error",
                "data": {
                    "message": "Target player is already dead"
                }
            });
            return Ok(serde_json::to_string(&response).map_err(|e| format!("Failed to serialize response: {}", e))?);
        }
        
        // 根据武器属性计算伤害（简化处理）
        let damage = 20; // 假设固定伤害值
        
        // 减少目标玩家生命值
        {
            let target_player = game_state.players.get_mut(&target_player_id).ok_or("Target player not found".to_string())?;
            target_player.life -= damage;
            
            // 检查目标玩家是否死亡
            if target_player.life <= 0 {
                target_player.life = 0;
                target_player.is_alive = false;
            }
        } // 释放对目标玩家的可变借用
        
        // 获取目标玩家的当前状态
        let (target_player_life, target_player_is_alive) = {
            let target_player = game_state.players.get(&target_player_id).ok_or("Target player not found".to_string())?;
            (target_player.life, target_player.is_alive)
        };
        
        // 向攻击者发送攻击结果（仅包括主目标）
        let attacker_response = json!({
            "type": "player_update",
            "data": {
                "message": format!("Attacked player {} for {} damage", target_player_id, damage),
                "target_player_life": target_player_life,
                "target_player_is_alive": target_player_is_alive
            }
        });
        
        // 向被攻击者发送被攻击通知（不包括攻击者身份）
        // 注意：这里我们不实际发送消息，只是构造响应
        let _target_response = json!({
            "type": "system_message",
            "data": {
                "message": format!("You were attacked for {} damage", damage)
            }
        });
        
        // 消耗体力值（根据规则配置，假设攻击消耗10点体力）
        let attack_cost = 10;
        {
            let player = game_state.players.get_mut(player_id).ok_or("Player not found".to_string())?;
            if player.strength >= attack_cost {
                player.strength -= attack_cost;
            } else {
                player.strength = 0;
            }
        } // 释放对攻击者的可变借用
        
        // 返回攻击者响应
        Ok(serde_json::to_string(&attacker_response).map_err(|e| format!("Failed to serialize response: {}", e))?)
    }

    async fn handle_equip_action(&self, game_state: &mut GameState, player_id: &str, action_data: serde_json::Value) -> Result<String, String> {
        // 获取玩家引用
        let player = game_state.players.get_mut(player_id).ok_or("Player not found".to_string())?;
        
        // 检查前置条件：玩家处于存活状态，背包中有指定物品
        if !player.is_alive {
            return Err("Player is not alive".to_string());
        }
        
        // 获取物品ID
        let item_id = action_data.get("item_id").and_then(|v| v.as_str())
            .ok_or("Missing item_id field")?;
        
        // 验证玩家背包中是否存在指定物品
        if let Some(_item_index) = player.inventory.iter().position(|item| item.id == item_id) {
            // 更新玩家当前手持物品
            player.hand_item = Some(item_id.to_string());
            
            // 向该玩家发送手持物品状态更新
            let response = json!({
                "type": "player_update",
                "data": {
                    "hand_item": item_id
                }
            });
            
            Ok(serde_json::to_string(&response).map_err(|e| format!("Failed to serialize response: {}", e))?)
        } else {
            Err("Item not found in player's inventory".to_string())
        }
    }

    async fn handle_use_action(&self, game_state: &mut GameState, player_id: &str, action_data: serde_json::Value) -> Result<String, String> {
        // 获取玩家引用
        let player = game_state.players.get_mut(player_id).ok_or("Player not found".to_string())?;
        
        // 检查前置条件：玩家处于存活状态，手持道具
        if !player.is_alive {
            return Err("Player is not alive".to_string());
        }
        
        if player.hand_item.is_none() {
            return Err("Player has no item in hand".to_string());
        }
        
        // 获取物品ID
        let item_id = action_data.get("item_id").and_then(|v| v.as_str())
            .ok_or("Missing item_id field")?;
        
        // 验证手持的是否是指定物品
        if player.hand_item.as_ref() != Some(&item_id.to_string()) {
            return Err("Specified item is not in player's hand".to_string());
        }
        
        // 查找物品信息
        if let Some(item_index) = player.inventory.iter().position(|item| item.id == item_id) {
            let item = &player.inventory[item_index];
            
            // 根据道具类型执行相应效果
            match item.item_type {
                ItemType::Consumable => {
                    // 如果是消耗品：恢复生命值、传送等，并从玩家背包中移除消耗品
                    // 这里我们简化处理，假设所有消耗品都恢复20点生命值
                    player.life += 20;
                    if player.life > 100 {
                        player.life = 100; // 假设最大生命值为100
                    }
                    
                    // 从玩家背包中移除消耗品
                    player.inventory.remove(item_index);
                    
                    // 清空手持物品
                    player.hand_item = None;
                    
                    // 更新玩家状态
                    let response = json!({
                        "type": "player_update",
                        "data": {
                            "life": player.life,
                            "inventory": player.inventory,
                            "hand_item": null
                        }
                    });
                    
                    Ok(serde_json::to_string(&response).map_err(|e| format!("Failed to serialize response: {}", e))?)
                }
                ItemType::Weapon => {
                    // 如果是装备类：装备到对应位置，替换原有装备
                    // 这里我们简化处理，只是更新装备状态
                    player.equipped_item = Some(item_id.to_string());
                    
                    // 更新玩家状态
                    let response = json!({
                        "type": "player_update",
                        "data": {
                            "equipped_item": item_id
                        }
                    });
                    
                    Ok(serde_json::to_string(&response).map_err(|e| format!("Failed to serialize response: {}", e))?)
                }
                ItemType::Equipment => {
                    // 其他装备类型处理
                    player.equipped_item = Some(item_id.to_string());
                    
                    // 更新玩家状态
                    let response = json!({
                        "type": "player_update",
                        "data": {
                            "equipped_item": item_id
                        }
                    });
                    
                    Ok(serde_json::to_string(&response).map_err(|e| format!("Failed to serialize response: {}", e))?)
                }
            }
        } else {
            Err("Item not found in player's inventory".to_string())
        }
    }

    async fn handle_throw_action(&self, game_state: &mut GameState, player_id: &str, action_data: serde_json::Value) -> Result<String, String> {
        // 获取玩家引用
        let player = game_state.players.get_mut(player_id).ok_or("Player not found".to_string())?;
        
        // 检查前置条件：玩家处于存活状态，背包中有指定物品
        if !player.is_alive {
            return Err("Player is not alive".to_string());
        }
        
        // 获取物品ID
        let item_id = action_data.get("item_id").and_then(|v| v.as_str())
            .ok_or("Missing item_id field")?;
        
        // 验证玩家背包中是否存在指定物品
        if let Some(item_index) = player.inventory.iter().position(|item| item.id == item_id) {
            // 从玩家背包中移除物品
            let item = player.inventory.remove(item_index);
            
            // 将物品添加到当前地点的物品列表
            if let Some(place) = game_state.places.get_mut(&player.location) {
                place.items.push(item);
            }
            
            // 如果丢弃的是手持物品，清空手持物品状态
            if player.hand_item.as_ref() == Some(&item_id.to_string()) {
                player.hand_item = None;
            }
            
            // 向该玩家发送背包更新
            let response = json!({
                "type": "player_update",
                "data": {
                    "inventory": player.inventory,
                    "hand_item": player.hand_item
                }
            });
            
            Ok(serde_json::to_string(&response).map_err(|e| format!("Failed to serialize response: {}", e))?)
        } else {
            Err("Item not found in player's inventory".to_string())
        }
    }

    async fn handle_deliver_action(&self, game_state: &mut GameState, player_id: &str, action_data: serde_json::Value) -> Result<String, String> {
        // 获取玩家引用
        let player = game_state.players.get_mut(player_id).ok_or("Player not found".to_string())?;
        
        // 检查前置条件：玩家处于存活状态
        if !player.is_alive {
            return Err("Player is not alive".to_string());
        }
        
        // 获取目标玩家ID和消息内容
        let target_player_id = action_data.get("target_player_id").and_then(|v| v.as_str())
            .ok_or("Missing target_player_id field")?;
        let message = action_data.get("message").and_then(|v| v.as_str())
            .ok_or("Missing message field")?;
        
        // 消耗体力值（根据规则配置，假设传音消耗5点体力）
        let deliver_cost = 5;
        if player.strength >= deliver_cost {
            player.strength -= deliver_cost;
        } else {
            player.strength = 0;
        }
        
        // 向目标玩家发送消息
        // 在实际实现中，这里需要找到目标玩家的连接并发送消息
        // 这里我们只是构造响应
        
        let response = json!({
            "type": "system_message",
            "data": {
                "message": format!("Delivered message to player {}: {}", target_player_id, message)
            }
        });
        
        Ok(serde_json::to_string(&response).map_err(|e| format!("Failed to serialize response: {}", e))?)
    }

    async fn handle_send_action(&self, _game_state: &mut GameState, player_id: &str, action_data: serde_json::Value) -> Result<String, String> {
        // 获取玩家引用
        let _player = _game_state.players.get_mut(player_id).ok_or("Player not found".to_string())?;
        
        // 获取消息内容
        let message = action_data.get("message").and_then(|v| v.as_str())
            .ok_or("Missing message field")?;
        
        // 将消息转发给导演客户端
        // 在实际实现中，这里需要找到导演的连接并发送消息
        // 这里我们只是构造响应
        
        let response = json!({
            "type": "system_message",
            "data": {
                "message": format!("Message sent to director: {}", message)
            }
        });
        
        Ok(serde_json::to_string(&response).map_err(|e| format!("Failed to serialize response: {}", e))?)
    }
}