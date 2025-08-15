use crate::models::admin_game::{
    CreateGameRequest, CreateGameResponse, CreateRuleTemplateRequest, CreateRuleTemplateResponse,
    DeleteGameResponse, DeleteRuleTemplateResponse, UpdateGameRequest, UpdateGameResponse,
    UpdateRuleTemplateRequest, UpdateRuleTemplateResponse,
};
use crate::services::admin_game_service;
use crate::services::db_helper::get_db_connection_from_pool;
use actix_web::{HttpResponse, Result, web};

// 创建游戏
pub async fn create_game(create_request: web::Json<CreateGameRequest>) -> Result<HttpResponse> {
    match get_db_connection_from_pool() {
        Ok(mut conn) => match admin_game_service::create_game(&mut conn, &create_request) {
            Ok(game_id) => Ok(HttpResponse::Ok().json(CreateGameResponse {
                success: true,
                game_id: Some(game_id),
                message: Some("Game created successfully".to_string()),
            })),
            Err(e) => {
                tracing::error!("Failed to create game: {}", e);
                Ok(
                    HttpResponse::InternalServerError().json(CreateGameResponse {
                        success: false,
                        game_id: None,
                        message: Some("Failed to create game".to_string()),
                    }),
                )
            }
        },
        Err(e) => {
            tracing::error!("Database connection error: {}", e);
            Ok(
                HttpResponse::InternalServerError().json(CreateGameResponse {
                    success: false,
                    game_id: None,
                    message: Some("Database connection error".to_string()),
                }),
            )
        }
    }
}

// 更新游戏
pub async fn update_game(
    path: web::Path<String>,
    update_request: web::Json<UpdateGameRequest>,
) -> Result<HttpResponse> {
    let game_id = path.into_inner();

    match get_db_connection_from_pool() {
        Ok(mut conn) => {
            match admin_game_service::update_game(&mut conn, &game_id, &update_request) {
                Ok(()) => Ok(HttpResponse::Ok().json(UpdateGameResponse {
                    success: true,
                    message: Some("Game updated successfully".to_string()),
                })),
                Err(e) => {
                    tracing::error!("Failed to update game: {}", e);
                    Ok(
                        HttpResponse::InternalServerError().json(UpdateGameResponse {
                            success: false,
                            message: Some("Failed to update game".to_string()),
                        }),
                    )
                }
            }
        }
        Err(e) => {
            tracing::error!("Database connection error: {}", e);
            Ok(
                HttpResponse::InternalServerError().json(UpdateGameResponse {
                    success: false,
                    message: Some("Database connection error".to_string()),
                }),
            )
        }
    }
}

// 删除游戏
pub async fn delete_game(path: web::Path<String>) -> Result<HttpResponse> {
    let game_id = path.into_inner();

    match get_db_connection_from_pool() {
        Ok(mut conn) => match admin_game_service::delete_game(&mut conn, &game_id) {
            Ok(()) => Ok(HttpResponse::Ok().json(DeleteGameResponse {
                success: true,
                message: Some("Game deleted successfully".to_string()),
            })),
            Err(e) => {
                tracing::error!("Failed to delete game: {}", e);
                Ok(
                    HttpResponse::InternalServerError().json(DeleteGameResponse {
                        success: false,
                        message: Some("Failed to delete game".to_string()),
                    }),
                )
            }
        },
        Err(e) => {
            tracing::error!("Database connection error: {}", e);
            Ok(
                HttpResponse::InternalServerError().json(DeleteGameResponse {
                    success: false,
                    message: Some("Database connection error".to_string()),
                }),
            )
        }
    }
}

// 创建规则模板
pub async fn create_rule_template(
    create_request: web::Json<CreateRuleTemplateRequest>,
) -> Result<HttpResponse> {
    match get_db_connection_from_pool() {
        Ok(mut conn) => {
            match admin_game_service::create_rule_template(&mut conn, &create_request) {
                Ok(template_id) => Ok(HttpResponse::Ok().json(CreateRuleTemplateResponse {
                    success: true,
                    template_id: Some(template_id),
                    message: Some("Rule template created successfully".to_string()),
                })),
                Err(e) => {
                    tracing::error!("Failed to create rule template: {}", e);
                    Ok(
                        HttpResponse::InternalServerError().json(CreateRuleTemplateResponse {
                            success: false,
                            template_id: None,
                            message: Some("Failed to create rule template".to_string()),
                        }),
                    )
                }
            }
        }
        Err(e) => {
            tracing::error!("Database connection error: {}", e);
            Ok(
                HttpResponse::InternalServerError().json(CreateRuleTemplateResponse {
                    success: false,
                    template_id: None,
                    message: Some("Database connection error".to_string()),
                }),
            )
        }
    }
}

// 更新规则模板
pub async fn update_rule_template(
    path: web::Path<String>,
    update_request: web::Json<UpdateRuleTemplateRequest>,
) -> Result<HttpResponse> {
    let template_id = path.into_inner();

    match get_db_connection_from_pool() {
        Ok(mut conn) => {
            match admin_game_service::update_rule_template(&mut conn, &template_id, &update_request)
            {
                Ok(()) => Ok(HttpResponse::Ok().json(UpdateRuleTemplateResponse {
                    success: true,
                    message: Some("Rule template updated successfully".to_string()),
                })),
                Err(e) => {
                    tracing::error!("Failed to update rule template: {}", e);
                    Ok(
                        HttpResponse::InternalServerError().json(UpdateRuleTemplateResponse {
                            success: false,
                            message: Some("Failed to update rule template".to_string()),
                        }),
                    )
                }
            }
        }
        Err(e) => {
            tracing::error!("Database connection error: {}", e);
            Ok(
                HttpResponse::InternalServerError().json(UpdateRuleTemplateResponse {
                    success: false,
                    message: Some("Database connection error".to_string()),
                }),
            )
        }
    }
}

// 删除规则模板
pub async fn delete_rule_template(path: web::Path<String>) -> Result<HttpResponse> {
    let template_id = path.into_inner();

    match get_db_connection_from_pool() {
        Ok(mut conn) => match admin_game_service::delete_rule_template(&mut conn, &template_id) {
            Ok(()) => Ok(HttpResponse::Ok().json(DeleteRuleTemplateResponse {
                success: true,
                message: Some("Rule template deleted successfully".to_string()),
            })),
            Err(e) => {
                tracing::error!("Failed to delete rule template: {}", e);
                Ok(
                    HttpResponse::InternalServerError().json(DeleteRuleTemplateResponse {
                        success: false,
                        message: Some("Failed to delete rule template".to_string()),
                    }),
                )
            }
        },
        Err(e) => {
            tracing::error!("Database connection error: {}", e);
            Ok(
                HttpResponse::InternalServerError().json(DeleteRuleTemplateResponse {
                    success: false,
                    message: Some("Database connection error".to_string()),
                }),
            )
        }
    }
}
