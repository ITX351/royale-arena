use actix::{Actor, ActorContext, StreamHandler};
use actix_web_actors::ws::{self, WebsocketContext};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct WsMessage {
    pub message_type: String,
    pub data: serde_json::Value,
}

/// WebSocket连接处理器
pub struct WsConnection {
    user_type: String, // "director" or "player"
    game_id: String,
    user_id: String, // player_id for players, "director" for directors
}

impl Actor for WsConnection {
    type Context = WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        tracing::info!(
            "WebSocket connection started for {} ({})",
            self.user_type,
            self.user_id
        );

        // 在实际实现中，我们需要：
        // 1. 验证连接权限
        // 2. 将连接添加到连接管理器中
        // 3. 发送初始状态给客户端

        // 发送连接成功的消息
        let response = WsMessage {
            message_type: "connection_success".to_string(),
            data: serde_json::json!({
                "message": "Connected successfully",
                "user_type": self.user_type,
                "game_id": self.game_id,
                "user_id": self.user_id
            }),
        };

        ctx.text(serde_json::to_string(&response).unwrap());
    }

    fn stopped(&mut self, _: &mut Self::Context) {
        tracing::info!(
            "WebSocket connection stopped for {} ({})",
            self.user_type,
            self.user_id
        );

        // 在实际实现中，我们需要：
        // 1. 从连接管理器中移除连接
        // 2. 处理连接断开的逻辑
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsConnection {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => {
                // 处理文本消息
                match serde_json::from_str::<WsMessage>(&text) {
                    Ok(ws_msg) => {
                        // 在实际实现中，我们需要根据消息类型处理不同的指令
                        match ws_msg.message_type.as_str() {
                            "player_action" => {
                                // 处理玩家行动指令
                                self.handle_player_action(ws_msg.data, ctx);
                            }
                            "director_command" => {
                                // 处理导演控制指令
                                self.handle_director_command(ws_msg.data, ctx);
                            }
                            "get_state" => {
                                // 处理获取状态请求
                                self.handle_get_state(ctx);
                            }
                            _ => {
                                // 未知消息类型
                                let response = WsMessage {
                                    message_type: "error".to_string(),
                                    data: serde_json::json!({
                                        "message": "Unknown message type"
                                    }),
                                };
                                ctx.text(serde_json::to_string(&response).unwrap());
                            }
                        }
                    }
                    Err(e) => {
                        // JSON解析错误
                        let response = WsMessage {
                            message_type: "error".to_string(),
                            data: serde_json::json!({
                                "message": format!("Invalid JSON: {}", e)
                            }),
                        };
                        ctx.text(serde_json::to_string(&response).unwrap());
                    }
                }
            }
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            _ => ctx.stop(),
        }
    }
}

impl WsConnection {
    pub fn new(user_type: String, game_id: String, user_id: String) -> Self {
        Self {
            user_type,
            game_id,
            user_id,
        }
    }

    fn handle_player_action(&self, data: serde_json::Value, ctx: &mut WebsocketContext<Self>) {
        // 在实际实现中，我们需要：
        // 1. 验证玩家权限
        // 2. 验证行动是否在允许的时间窗口内
        // 3. 处理具体的行动指令（移动、搜索、攻击、使用道具等）
        // 4. 更新游戏状态
        // 5. 广播结果给所有连接的客户端

        tracing::info!("Handling player action: {:?}", data);

        // 示例响应
        let response = WsMessage {
            message_type: "action_result".to_string(),
            data: serde_json::json!({
                "success": true,
                "message": "Action processed successfully",
                "data": data
            }),
        };

        ctx.text(serde_json::to_string(&response).unwrap());
    }

    fn handle_director_command(&self, data: serde_json::Value, ctx: &mut WebsocketContext<Self>) {
        // 在实际实现中，我们需要：
        // 1. 验证导演权限
        // 2. 处理导演控制指令（开始行动、结束行动、缩圈、空投等）
        // 3. 更新游戏状态
        // 4. 广播结果给所有连接的客户端

        tracing::info!("Handling director command: {:?}", data);

        // 示例响应
        let response = WsMessage {
            message_type: "command_result".to_string(),
            data: serde_json::json!({
                "success": true,
                "message": "Command processed successfully",
                "data": data
            }),
        };

        ctx.text(serde_json::to_string(&response).unwrap());
    }

    fn handle_get_state(&self, ctx: &mut WebsocketContext<Self>) {
        // 在实际实现中，我们需要：
        // 1. 获取当前游戏状态
        // 2. 发送状态给请求的客户端

        tracing::info!("Handling get state request");

        // 示例响应
        let response = WsMessage {
            message_type: "game_state".to_string(),
            data: serde_json::json!({
                "game_id": self.game_id,
                "status": "running",
                "phase": "day",
                "players": 50,
                "alive": 30
            }),
        };

        ctx.text(serde_json::to_string(&response).unwrap());
    }
}
