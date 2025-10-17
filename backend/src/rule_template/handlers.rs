use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde_json::json;

use super::{
    errors::RuleTemplateError,
    models::{CreateRuleTemplateRequest, GetTemplatesQuery, UpdateRuleTemplateRequest},
};
use crate::routes::AppState;

/// 创建游戏规则模版
/// POST /api/admin/rule-templates
pub async fn create_template(
    State(app_state): State<AppState>,
    Json(request): Json<CreateRuleTemplateRequest>,
) -> Result<impl IntoResponse, RuleTemplateError> {
    let template = app_state
        .rule_template_service
        .create_template(request)
        .await?;

    let response = Json(json!({
        "success": true,
        "data": template
    }));

    Ok((StatusCode::CREATED, response))
}

/// 更新游戏规则模版
/// PUT /api/admin/rule-templates/{template_id}
pub async fn update_template(
    State(app_state): State<AppState>,
    Path(template_id): Path<String>,
    Json(request): Json<UpdateRuleTemplateRequest>,
) -> Result<impl IntoResponse, RuleTemplateError> {
    let template = app_state
        .rule_template_service
        .update_template(template_id, request)
        .await?;

    let response = Json(json!({
        "success": true,
        "data": template
    }));

    Ok((StatusCode::OK, response))
}

/// 获取游戏规则模版（统一接口）
/// GET /api/rule-templates
/// 支持查询参数：id, is_active, search
pub async fn get_templates(
    State(app_state): State<AppState>,
    Query(params): Query<GetTemplatesQuery>,
) -> Result<impl IntoResponse, RuleTemplateError> {
    let templates = app_state
        .rule_template_service
        .get_templates(params.id, params.is_active, params.search)
        .await?;

    let response = Json(json!({
        "success": true,
        "data": templates
    }));

    Ok((StatusCode::OK, response))
}
