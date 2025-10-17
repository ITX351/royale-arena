use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// 数据库实体：规则模版
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RuleTemplate {
    pub id: String, // VARCHAR(36) in database
    pub template_name: String,
    pub description: Option<String>, // TEXT type in database
    pub is_active: bool,
    pub rules_config: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// API响应：规则模版详情（统一格式）
#[derive(Debug, Serialize, Deserialize)]
pub struct RuleTemplateResponse {
    pub id: String,
    pub template_name: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub rules_config: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// API请求：创建规则模版
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateRuleTemplateRequest {
    pub template_name: String,
    pub description: Option<String>,
    pub is_active: Option<bool>,
    pub rules_config: serde_json::Value,
}

/// API请求：更新规则模版
#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateRuleTemplateRequest {
    pub template_name: Option<String>,
    pub description: Option<String>,
    pub is_active: Option<bool>,
    pub rules_config: Option<serde_json::Value>,
}

impl From<RuleTemplate> for RuleTemplateResponse {
    fn from(template: RuleTemplate) -> Self {
        Self {
            id: template.id,
            template_name: template.template_name,
            description: template.description,
            is_active: template.is_active,
            rules_config: template.rules_config,
            created_at: template.created_at,
            updated_at: template.updated_at,
        }
    }
}

impl CreateRuleTemplateRequest {
    /// 验证请求数据
    pub fn validate(&self) -> Result<(), String> {
        if self.template_name.trim().is_empty() {
            return Err("模版名称不能为空".to_string());
        }

        if self.template_name.len() > 100 {
            return Err("模版名称不能超过100个字符".to_string());
        }

        // 验证 rules_config 是否为有效的对象
        if !self.rules_config.is_object() {
            return Err("规则配置必须是有效的JSON对象".to_string());
        }

        Ok(())
    }
}

impl UpdateRuleTemplateRequest {
    /// 验证请求数据
    pub fn validate(&self) -> Result<(), String> {
        // 至少需要提供一个字段
        if self.template_name.is_none()
            && self.description.is_none()
            && self.is_active.is_none()
            && self.rules_config.is_none()
        {
            return Err("至少需要提供一个要更新的字段".to_string());
        }

        if let Some(ref name) = self.template_name {
            if name.trim().is_empty() {
                return Err("模版名称不能为空".to_string());
            }

            if name.len() > 100 {
                return Err("模版名称不能超过100个字符".to_string());
            }
        }

        if let Some(ref config) = self.rules_config {
            if !config.is_object() {
                return Err("规则配置必须是有效的JSON对象".to_string());
            }
        }

        Ok(())
    }
}

/// 查询参数结构
#[derive(Debug, Deserialize)]
pub struct GetTemplatesQuery {
    pub id: Option<String>,
    pub is_active: Option<bool>,
    pub search: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_create_template_validation() {
        // 正常请求应该验证通过
        let valid_request = CreateRuleTemplateRequest {
            template_name: "测试模版".to_string(),
            description: Some("测试描述".to_string()),
            is_active: Some(true),
            rules_config: json!({
                "game_flow": {
                    "day_duration": 300
                }
            }),
        };
        assert!(valid_request.validate().is_ok());

        // 空名称应该失败
        let empty_name_request = CreateRuleTemplateRequest {
            template_name: "".to_string(),
            description: None,
            is_active: Some(true),
            rules_config: json!({"test": true}),
        };
        assert!(empty_name_request.validate().is_err());
        assert_eq!(
            empty_name_request.validate().unwrap_err(),
            "模版名称不能为空"
        );

        // 过长名称应该失败
        let long_name_request = CreateRuleTemplateRequest {
            template_name: "a".repeat(101),
            description: None,
            is_active: Some(true),
            rules_config: json!({"test": true}),
        };
        assert!(long_name_request.validate().is_err());
        assert_eq!(
            long_name_request.validate().unwrap_err(),
            "模版名称不能超过100个字符"
        );

        // 非对象配置应该失败
        let invalid_config_request = CreateRuleTemplateRequest {
            template_name: "测试".to_string(),
            description: None,
            is_active: Some(true),
            rules_config: json!("not an object"),
        };
        assert!(invalid_config_request.validate().is_err());
        assert_eq!(
            invalid_config_request.validate().unwrap_err(),
            "规则配置必须是有效的JSON对象"
        );
    }

    #[test]
    fn test_update_template_validation() {
        // 正常更新请求应该验证通过
        let valid_update = UpdateRuleTemplateRequest {
            template_name: Some("更新名称".to_string()),
            description: Some("更新描述".to_string()),
            is_active: Some(false),
            rules_config: Some(json!({"updated": true})),
        };
        assert!(valid_update.validate().is_ok());

        // 空更新请求应该失败
        let empty_update = UpdateRuleTemplateRequest {
            template_name: None,
            description: None,
            is_active: None,
            rules_config: None,
        };
        assert!(empty_update.validate().is_err());
        assert_eq!(
            empty_update.validate().unwrap_err(),
            "至少需要提供一个要更新的字段"
        );

        // 空名称更新应该失败
        let empty_name_update = UpdateRuleTemplateRequest {
            template_name: Some("".to_string()),
            description: None,
            is_active: None,
            rules_config: None,
        };
        assert!(empty_name_update.validate().is_err());
        assert_eq!(
            empty_name_update.validate().unwrap_err(),
            "模版名称不能为空"
        );

        // 过长名称更新应该失败
        let long_name_update = UpdateRuleTemplateRequest {
            template_name: Some("a".repeat(101)),
            description: None,
            is_active: None,
            rules_config: None,
        };
        assert!(long_name_update.validate().is_err());
        assert_eq!(
            long_name_update.validate().unwrap_err(),
            "模版名称不能超过100个字符"
        );

        // 非对象配置更新应该失败
        let invalid_config_update = UpdateRuleTemplateRequest {
            template_name: None,
            description: None,
            is_active: None,
            rules_config: Some(json!("not an object")),
        };
        assert!(invalid_config_update.validate().is_err());
        assert_eq!(
            invalid_config_update.validate().unwrap_err(),
            "规则配置必须是有效的JSON对象"
        );
    }

    #[test]
    fn test_rule_template_response_conversion() {
        // 测试从RuleTemplate到RuleTemplateResponse的转换
        let template = RuleTemplate {
            id: "test-id".to_string(),
            template_name: "测试模版".to_string(),
            description: Some("测试描述".to_string()),
            is_active: true,
            rules_config: json!({"test": "config"}),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let response: RuleTemplateResponse = template.clone().into();
        assert_eq!(response.id, template.id);
        assert_eq!(response.template_name, template.template_name);
        assert_eq!(response.description, template.description);
        assert_eq!(response.is_active, template.is_active);
        assert_eq!(response.rules_config, template.rules_config);
    }
}
