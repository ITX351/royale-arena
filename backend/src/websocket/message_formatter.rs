//! WebSocket消息格式化模块
//!
//! 该模块提供统一的消息生成和格式化功能，用于WebSocket通信。
//! 使用方法：
//! 1. 调用相应的消息生成函数（如system_message, game_state_message等）直接生成可发送的WebSocket消息

use axum::extract::ws::{Message, Utf8Bytes};
use serde_json::{Value as JsonValue, json};

/// 内部消息生成函数，直接返回可发送的WebSocket消息
fn generate_message(message_type: &str, data: JsonValue) -> Message {
    let json_value = json!({
        "type": message_type,
        "data": data
    });
    Message::Text(Utf8Bytes::from(serde_json::to_string(&json_value).unwrap()))
}

/// 生成系统消息，直接返回可发送的WebSocket消息
pub fn system_message(data: JsonValue) -> Message {
    generate_message("system_message", data)
}

/// 生成游戏状态消息，直接返回可发送的WebSocket消息
pub fn game_state_message(data: JsonValue) -> Message {
    generate_message("game_state", data)
}

/// 生成错误消息，直接返回可发送的WebSocket消息
pub fn error_message(data: JsonValue) -> Message {
    generate_message("error", data)
}
