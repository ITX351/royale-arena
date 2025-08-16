use actix_web::{HttpResponse, Result, web};
use serde::{Deserialize, Serialize};
use crate::services::actor_service::{get_actor_from_db, verify_director_password};

#[derive(Serialize, Deserialize)]
pub struct WsAuthRequest {
    pub game_id: String,
    pub password: String,          // 可以是导演密码或演员密码
    pub user_type: String,         // "director" or "player"
    pub player_id: Option<String>, // 如果是玩家，需要提供player_id
}

#[derive(Serialize, Deserialize)]
pub struct WsAuthResponse {
    pub success: bool,
    pub message: String,
    pub token: Option<String>,
}

// 验证WebSocket连接凭据
pub async fn ws_auth(
    req: web::Json<WsAuthRequest>,
) -> Result<HttpResponse> {
    let auth_request = req.into_inner();
    
    // 验证必填字段
    if auth_request.game_id.is_empty() {
        return Ok(HttpResponse::BadRequest().json(WsAuthResponse {
            success: false,
            message: "游戏ID不能为空".to_string(),
            token: None,
        }));
    }
    
    if auth_request.password.is_empty() {
        return Ok(HttpResponse::BadRequest().json(WsAuthResponse {
            success: false,
            message: "密码不能为空".to_string(),
            token: None,
        }));
    }
    
    if auth_request.user_type.is_empty() {
        return Ok(HttpResponse::BadRequest().json(WsAuthResponse {
            success: false,
            message: "用户类型不能为空".to_string(),
            token: None,
        }));
    }
    
    // 验证用户类型
    if auth_request.user_type != "director" && auth_request.user_type != "player" {
        return Ok(HttpResponse::BadRequest().json(WsAuthResponse {
            success: false,
            message: "用户类型必须是director或player".to_string(),
            token: None,
        }));
    }
    
    // 根据用户类型进行不同的验证
    match auth_request.user_type.as_str() {
        "director" => {
            // 验证导演密码
            match verify_director_password(&auth_request.game_id, &auth_request.password).await {
                Ok(valid) => {
                    if valid {
                        // 生成临时访问令牌
                        let token = uuid::Uuid::new_v4().to_string();
                        return Ok(HttpResponse::Ok().json(WsAuthResponse {
                            success: true,
                            message: "认证成功".to_string(),
                            token: Some(token),
                        }));
                    } else {
                        return Ok(HttpResponse::Unauthorized().json(WsAuthResponse {
                            success: false,
                            message: "导演密码错误".to_string(),
                            token: None,
                        }));
                    }
                }
                Err(e) => {
                    tracing::error!("验证导演密码时发生错误: {}", e);
                    return Ok(HttpResponse::InternalServerError().json(WsAuthResponse {
                        success: false,
                        message: "服务器内部错误".to_string(),
                        token: None,
                    }));
                }
            }
        }
        "player" => {
            // 检查是否提供了player_id
            let player_id = match &auth_request.player_id {
                Some(id) if !id.is_empty() => id,
                _ => {
                    return Ok(HttpResponse::BadRequest().json(WsAuthResponse {
                        success: false,
                        message: "玩家类型必须提供player_id".to_string(),
                        token: None,
                    }));
                }
            };
            
            // 获取演员信息并验证密码
            match get_actor_from_db(&auth_request.game_id, player_id).await {
                Ok(Some(actor)) => {
                    if actor.verify_password(&auth_request.password) {
                        // 生成临时访问令牌
                        let token = uuid::Uuid::new_v4().to_string();
                        return Ok(HttpResponse::Ok().json(WsAuthResponse {
                            success: true,
                            message: "认证成功".to_string(),
                            token: Some(token),
                        }));
                    } else {
                        return Ok(HttpResponse::Unauthorized().json(WsAuthResponse {
                            success: false,
                            message: "演员密码错误".to_string(),
                            token: None,
                        }));
                    }
                }
                Ok(None) => {
                    return Ok(HttpResponse::Unauthorized().json(WsAuthResponse {
                        success: false,
                        message: "未找到指定的演员".to_string(),
                        token: None,
                    }));
                }
                Err(e) => {
                    tracing::error!("获取演员信息时发生错误: {}", e);
                    return Ok(HttpResponse::InternalServerError().json(WsAuthResponse {
                        success: false,
                        message: "服务器内部错误".to_string(),
                        token: None,
                    }));
                }
            }
        }
        _ => {
            // 这个情况实际上已经在前面检查过了，但为了完整性还是加上
            return Ok(HttpResponse::BadRequest().json(WsAuthResponse {
                success: false,
                message: "用户类型必须是director或player".to_string(),
                token: None,
            }));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::web;
    use crate::test_init::init_test_env;

    #[actix_web::test]
    async fn test_ws_auth_director_success() {
        // Initialize test environment
        init_test_env();
        
        // Create request data for director
        let request_data = WsAuthRequest {
            game_id: "game-001".to_string(),  // 使用测试数据中的游戏ID
            password: "director123".to_string(),  // 使用测试数据中的导演密码
            user_type: "director".to_string(),
            player_id: None,
        };

        // Make request directly to the handler function
        let json = web::Json(request_data);
        let result = ws_auth(json).await;
        
        // We expect a successful response
        assert!(result.is_ok());
        
        // Check that we get a proper HTTP response
        let resp = result.unwrap();
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_ws_auth_player_success() {
        // Initialize test environment
        init_test_env();
        
        // Create request data for player
        let request_data = WsAuthRequest {
            game_id: "game-001".to_string(),  // 使用测试数据中的游戏ID
            password: "actor1".to_string(),   // 使用测试数据中的演员密码
            user_type: "player".to_string(),
            player_id: Some("actor-001".to_string()),  // 使用测试数据中的演员ID
        };

        // Make request directly to the handler function
        let json = web::Json(request_data);
        let result = ws_auth(json).await;
        
        // We expect a successful response
        assert!(result.is_ok());
        
        // Check that we get a proper HTTP response
        let resp = result.unwrap();
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_ws_auth_director_invalid_password() {
        // Initialize test environment
        init_test_env();
        
        // Create request data for director with invalid password
        let request_data = WsAuthRequest {
            game_id: "game-001".to_string(),
            password: "wrongpassword".to_string(),
            user_type: "director".to_string(),
            player_id: None,
        };

        // Make request directly to the handler function
        let json = web::Json(request_data);
        let result = ws_auth(json).await;
        
        // We expect a successful response
        assert!(result.is_ok());
        
        // Check that we get an unauthorized response
        let resp = result.unwrap();
        assert_eq!(resp.status(), actix_web::http::StatusCode::UNAUTHORIZED);
    }

    #[actix_web::test]
    async fn test_ws_auth_player_invalid_password() {
        // Initialize test environment
        init_test_env();
        
        // Create request data for player with invalid password
        let request_data = WsAuthRequest {
            game_id: "game-001".to_string(),
            password: "wrongpassword".to_string(),
            user_type: "player".to_string(),
            player_id: Some("actor-001".to_string()),
        };

        // Make request directly to the handler function
        let json = web::Json(request_data);
        let result = ws_auth(json).await;
        
        // We expect a successful response
        assert!(result.is_ok());
        
        // Check that we get an unauthorized response
        let resp = result.unwrap();
        assert_eq!(resp.status(), actix_web::http::StatusCode::UNAUTHORIZED);
    }

    #[actix_web::test]
    async fn test_ws_auth_invalid_user_type() {
        // Initialize test environment
        init_test_env();
        
        // Create request data with invalid user_type
        let request_data = WsAuthRequest {
            game_id: "game-001".to_string(),
            password: "director123".to_string(),
            user_type: "invalid".to_string(),
            player_id: None,
        };

        // Make request directly to the handler function
        let json = web::Json(request_data);
        let result = ws_auth(json).await;
        
        // We expect a successful response
        assert!(result.is_ok());
        
        // Check that we get a bad request response
        let resp = result.unwrap();
        assert_eq!(resp.status(), actix_web::http::StatusCode::BAD_REQUEST);
    }

    #[actix_web::test]
    async fn test_ws_auth_player_missing_player_id() {
        // Initialize test environment
        init_test_env();
        
        // Create request data for player without player_id
        let request_data = WsAuthRequest {
            game_id: "game-001".to_string(),
            password: "actor1".to_string(),
            user_type: "player".to_string(),
            player_id: None,
        };

        // Make request directly to the handler function
        let json = web::Json(request_data);
        let result = ws_auth(json).await;
        
        // We expect a successful response
        assert!(result.is_ok());
        
        // Check that we get a bad request response
        let resp = result.unwrap();
        assert_eq!(resp.status(), actix_web::http::StatusCode::BAD_REQUEST);
    }

    #[actix_web::test]
    async fn test_ws_auth_missing_game_id() {
        // Initialize test environment
        init_test_env();
        
        // Create request data with missing game_id
        let request_data = WsAuthRequest {
            game_id: "".to_string(),
            password: "director123".to_string(),
            user_type: "director".to_string(),
            player_id: None,
        };

        // Make request directly to the handler function
        let json = web::Json(request_data);
        let result = ws_auth(json).await;
        
        // We expect a successful response
        assert!(result.is_ok());
        
        // Check that we get a bad request response
        let resp = result.unwrap();
        assert_eq!(resp.status(), actix_web::http::StatusCode::BAD_REQUEST);
    }
}