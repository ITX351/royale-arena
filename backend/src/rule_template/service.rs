use sqlx::MySqlPool;
use uuid::Uuid;

use super::{
    errors::RuleTemplateError,
    models::{CreateRuleTemplateRequest, RuleTemplate, RuleTemplateResponse, UpdateRuleTemplateRequest},
};

#[derive(Debug, Clone)]
pub struct RuleTemplateService {
    pool: MySqlPool,
}

impl RuleTemplateService {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    /// 创建规则模版
    pub async fn create_template(
        &self,
        request: CreateRuleTemplateRequest,
    ) -> Result<RuleTemplateResponse, RuleTemplateError> {
        // 验证请求数据
        request.validate()
            .map_err(RuleTemplateError::ValidationError)?;

        // 检查模版名称唯一性
        if !self.check_name_uniqueness(&request.template_name, None).await? {
            return Err(RuleTemplateError::NameAlreadyExists);
        }

        // 生成UUID
        let template_id = Uuid::new_v4().to_string();
        let is_active = request.is_active.unwrap_or(true);

        // 插入数据库
        sqlx::query(
            r#"
            INSERT INTO rule_templates (id, template_name, description, is_active, rules_config)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(&template_id)
        .bind(&request.template_name)
        .bind(&request.description)
        .bind(is_active)
        .bind(&request.rules_config)
        .execute(&self.pool)
        .await?;

        // 查询创建的模版并返回
        let template = self.get_template_by_id(&template_id).await?
            .ok_or(RuleTemplateError::TemplateNotFound)?;

        Ok(template.into())
    }

    /// 更新规则模版
    pub async fn update_template(
        &self,
        template_id: String,
        request: UpdateRuleTemplateRequest,
    ) -> Result<RuleTemplateResponse, RuleTemplateError> {
        // 验证请求数据
        request.validate()
            .map_err(RuleTemplateError::ValidationError)?;

        // 检查模版是否存在
        if !self.check_template_exists(&template_id).await? {
            return Err(RuleTemplateError::TemplateNotFound);
        }

        // 如果要更新名称，检查名称唯一性
        if let Some(ref name) = request.template_name {
            if !self.check_name_uniqueness(name, Some(&template_id)).await? {
                return Err(RuleTemplateError::NameAlreadyExists);
            }
        }

        // 构建动态更新SQL
        let mut update_fields = Vec::new();

        if request.template_name.is_some() {
            update_fields.push("template_name = ?");
        }
        if request.description.is_some() {
            update_fields.push("description = ?");
        }
        if request.is_active.is_some() {
            update_fields.push("is_active = ?");
        }
        if request.rules_config.is_some() {
            update_fields.push("rules_config = ?");
        }

        if update_fields.is_empty() {
            return Err(RuleTemplateError::ValidationError("没有要更新的字段".to_string()));
        }

        update_fields.push("updated_at = CURRENT_TIMESTAMP");

        let sql = format!(
            "UPDATE rule_templates SET {} WHERE id = ?",
            update_fields.join(", ")
        );

        let mut query = sqlx::query(&sql);

        if let Some(ref name) = request.template_name {
            query = query.bind(name);
        }
        if let Some(ref desc) = request.description {
            query = query.bind(desc);
        }
        if let Some(active) = request.is_active {
            query = query.bind(active);
        }
        if let Some(ref config) = request.rules_config {
            query = query.bind(config);
        }

        query = query.bind(&template_id);

        query.execute(&self.pool).await?;

        // 查询更新后的模版并返回
        let template = self.get_template_by_id(&template_id).await?
            .ok_or(RuleTemplateError::TemplateNotFound)?;

        Ok(template.into())
    }

    /// 获取规则模版列表（统一接口）
    pub async fn get_templates(
        &self,
        id: Option<String>,
        is_active: Option<bool>,
        search: Option<String>,
    ) -> Result<Vec<RuleTemplateResponse>, RuleTemplateError> {
        // 如果提供了ID，优先按ID查询
        if let Some(ref template_id) = id {
            let query = sqlx::query_as::<_, RuleTemplate>(
                "SELECT id, template_name, description, is_active, rules_config, created_at, updated_at FROM rule_templates WHERE id = ?"
            ).bind(template_id);
            
            let templates = query.fetch_all(&self.pool).await?;
            return Ok(templates.into_iter().map(|t| t.into()).collect());
        }

        // 构建条件查询
        let mut conditions = Vec::new();
        if is_active.is_some() {
            conditions.push("is_active = ?");
        }
        if search.is_some() {
            conditions.push("template_name LIKE ?");
        }

        let base_sql = "SELECT id, template_name, description, is_active, rules_config, created_at, updated_at FROM rule_templates";
        
        let templates = if conditions.is_empty() {
            let sql = format!("{} ORDER BY created_at DESC", base_sql);
            sqlx::query_as::<_, RuleTemplate>(&sql)
                .fetch_all(&self.pool)
                .await?
        } else {
            let where_clause = format!(" WHERE {}", conditions.join(" AND "));
            let sql = format!("{}{} ORDER BY created_at DESC", base_sql, where_clause);
            
            let mut query = sqlx::query_as::<_, RuleTemplate>(&sql);
            
            if let Some(active) = is_active {
                query = query.bind(active);
            }
            if let Some(ref search_term) = search {
                query = query.bind(format!("%{}%", search_term));
            }
            
            query.fetch_all(&self.pool).await?
        };
        
        Ok(templates.into_iter().map(|t| t.into()).collect())
    }

    /// 根据ID获取单个模版
    async fn get_template_by_id(&self, template_id: &str) -> Result<Option<RuleTemplate>, RuleTemplateError> {
        let template = sqlx::query_as::<_, RuleTemplate>(
            "SELECT id, template_name, description, is_active, rules_config, created_at, updated_at FROM rule_templates WHERE id = ?"
        )
        .bind(template_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(template)
    }

    /// 检查模版是否存在
    async fn check_template_exists(&self, template_id: &str) -> Result<bool, sqlx::Error> {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM rule_templates WHERE id = ?")
            .bind(template_id)
            .fetch_one(&self.pool)
            .await?;

        Ok(count.0 > 0)
    }

    /// 检查模版名称唯一性
    async fn check_name_uniqueness(
        &self,
        name: &str,
        exclude_id: Option<&str>,
    ) -> Result<bool, sqlx::Error> {
        let count: (i64,) = if let Some(id) = exclude_id {
            sqlx::query_as("SELECT COUNT(*) FROM rule_templates WHERE template_name = ? AND id != ?")
                .bind(name)
                .bind(id)
                .fetch_one(&self.pool)
                .await?
        } else {
            sqlx::query_as("SELECT COUNT(*) FROM rule_templates WHERE template_name = ?")
                .bind(name)
                .fetch_one(&self.pool)
                .await?
        };

        Ok(count.0 == 0)
    }
}