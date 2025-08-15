use actix_web::{web, HttpResponse, Result};
use serde_json::json;
use crate::services::game_service::get_rule_template_from_db;

pub async fn get_rule_templates(
    data: web::Data<std::sync::Arc<tokio::sync::Mutex<crate::AppState>>>
) -> Result<HttpResponse> {
    let state = data.lock().await;
    
    // 获取所有规则模版
    let templates: Vec<&crate::models::rule_template::RuleTemplate> = 
        state.rule_templates.values().collect();
    
    // 如果没有自定义模版，返回默认模版
    if templates.is_empty() {
        let default_templates = vec![
            crate::models::rule_template::RuleTemplate::default_template(),
            crate::models::rule_template::RuleTemplate::quick_game_template(),
            crate::models::rule_template::RuleTemplate::endurance_template(),
        ];
        return Ok(HttpResponse::Ok().json(default_templates));
    }
    
    Ok(HttpResponse::Ok().json(templates))
}

pub async fn get_rule_template(
    path: web::Path<String>,
    data: web::Data<std::sync::Arc<tokio::sync::Mutex<crate::AppState>>>
) -> Result<HttpResponse> {
    let template_id = path.into_inner();
    
    // 首先尝试从内存状态中获取
    {
        let state = data.lock().await;
        if let Some(template) = state.rule_templates.get(&template_id) {
            return Ok(HttpResponse::Ok().json(template));
        }
    }
    
    // 如果没有找到，检查是否是默认模版
    let default_template = match template_id.as_str() {
        "default" => Some(crate::models::rule_template::RuleTemplate::default_template()),
        "quick" => Some(crate::models::rule_template::RuleTemplate::quick_game_template()),
        "endurance" => Some(crate::models::rule_template::RuleTemplate::endurance_template()),
        _ => None,
    };
    
    if let Some(template) = default_template {
        return Ok(HttpResponse::Ok().json(template));
    }
    
    // 最后尝试从数据库获取
    match get_rule_template_from_db(&template_id) {
        Ok(Some(rules)) => {
            let template = crate::models::rule_template::RuleTemplate::new(
                template_id.clone(),
                format!("数据库模板: {}", template_id),
                "从数据库加载的规则模板".to_string(),
                rules,
            );
            Ok(HttpResponse::Ok().json(template))
        },
        Ok(None) => {
            // 如果都没找到，返回404
            Ok(HttpResponse::NotFound().json(json!({
                "error": "Rule template not found"
            })))
        },
        Err(_) => {
            // 如果数据库错误，也返回404而不是500
            Ok(HttpResponse::NotFound().json(json!({
                "error": "Rule template not found"
            })))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, web};
    use serde_json::Value;
    use crate::test_utils::{create_test_app, create_test_app_state};

    #[actix_web::test]
    async fn test_get_rule_templates_default() {
        // Create test app state and app
        let app_state = create_test_app_state();
        let app = test::init_service(
            create_test_app(app_state.clone())
                .route("/rule-templates", web::get().to(get_rule_templates))
        ).await;

        // Make request for rule templates
        let req = test::TestRequest::get().uri("/rule-templates").to_request();
        let resp = test::call_service(&app, req).await;

        // Check response
        assert!(resp.status().is_success());
        
        let body = test::read_body(resp).await;
        let json: Value = serde_json::from_slice(&body).unwrap();
        
        // Check that we got an array of templates
        assert!(json.as_array().is_some());
        assert!(!json.as_array().unwrap().is_empty());
    }

    #[actix_web::test]
    async fn test_get_rule_template_default() {
        // Create test app state and app
        let app_state = create_test_app_state();
        let app = test::init_service(
            create_test_app(app_state.clone())
                .route("/rule-templates/{template_id}", web::get().to(get_rule_template))
        ).await;

        // Make request for a default rule template
        let req = test::TestRequest::get().uri("/rule-templates/default").to_request();
        let resp = test::call_service(&app, req).await;

        // Check response
        assert!(resp.status().is_success());
        
        let body = test::read_body(resp).await;
        let json: Value = serde_json::from_slice(&body).unwrap();
        
        // Check that we got a valid template object
        assert_eq!(json["id"], "default");
        assert_eq!(json["name"], "默认规则");
    }

    #[actix_web::test]
    async fn test_get_rule_template_not_found() {
        // Create test app state and app
        let app_state = create_test_app_state();
        let app = test::init_service(
            create_test_app(app_state.clone())
                .route("/rule-templates/{template_id}", web::get().to(get_rule_template))
        ).await;

        // Make request for a non-existent rule template
        let req = test::TestRequest::get().uri("/rule-templates/nonexistent").to_request();
        let resp = test::call_service(&app, req).await;

        // Check response
        assert_eq!(resp.status(), actix_web::http::StatusCode::NOT_FOUND);
    }
}