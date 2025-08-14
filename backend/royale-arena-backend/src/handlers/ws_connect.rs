use actix_web::{web, HttpRequest, HttpResponse, Result};
use actix_web_actors::ws;

use crate::handlers::websocket::WsConnection;

// WebSocket连接端点
pub async fn ws_connect(
    req: HttpRequest,
    stream: web::Payload,
    path: web::Path<(String, String)>, // (game_id, user_type)
) -> Result<HttpResponse> {
    let (game_id, user_type) = path.into_inner();
    
    // 在实际实现中，我们需要：
    // 1. 验证连接参数
    // 2. 从查询参数或headers中获取用户标识
    // 3. 创建WebSocket连接处理器
    
    // 从查询参数中获取用户ID（示例：?user_id=player123）
    let user_id = req.query_string()
        .split('&')
        .find_map(|pair| {
            let mut parts = pair.split('=');
            if parts.next() == Some("user_id") {
                parts.next()
            } else {
                None
            }
        })
        .unwrap_or("unknown")
        .to_string();
    
    // 创建WebSocket连接处理器
    let ws_actor = WsConnection::new(user_type, game_id, user_id);
    
    // 启动WebSocket连接
    let resp = ws::start(ws_actor, &req, stream);
    tracing::info!("WebSocket connection attempt: {:?} {:?}", req, resp);
    resp
}