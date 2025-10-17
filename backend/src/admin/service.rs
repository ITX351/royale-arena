use crate::admin::models::{AdminUser, AdminUserResponse, CreateAdminRequest, UpdateAdminRequest};
use crate::database::DatabasePool;
use crate::errors::ServiceError;
use uuid::Uuid;

#[derive(Clone)]
pub struct AdminService {
    pool: DatabasePool,
    bcrypt_cost: u32,
}

impl AdminService {
    pub fn new(pool: DatabasePool, bcrypt_cost: u32) -> Self {
        Self { pool, bcrypt_cost }
    }

    pub async fn list_admins(&self) -> Result<Vec<AdminUserResponse>, ServiceError> {
        let admins = sqlx::query_as::<_, AdminUser>(
            r#"
            SELECT id, username, password, is_super_admin, created_at, updated_at 
            FROM admin_users 
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(admins.into_iter().map(AdminUserResponse::from).collect())
    }

    pub async fn create_admin(
        &self,
        request: CreateAdminRequest,
    ) -> Result<AdminUserResponse, ServiceError> {
        // 检查用户名是否已存在
        if self.username_exists(&request.username).await? {
            return Err(ServiceError::UserAlreadyExists);
        }

        // 生成UUID和加密密码
        let id = Uuid::new_v4().to_string();
        let hashed_password = bcrypt::hash(&request.password, self.bcrypt_cost)?;

        // 插入新管理员
        sqlx::query(
            r#"
            INSERT INTO admin_users (id, username, password, is_super_admin)
            VALUES (?, ?, ?, ?)
            "#,
        )
        .bind(&id)
        .bind(&request.username)
        .bind(&hashed_password)
        .bind(request.is_super_admin)
        .execute(&self.pool)
        .await?;

        // 返回创建的用户信息
        Ok(AdminUserResponse {
            id,
            username: request.username,
            is_super_admin: request.is_super_admin,
        })
    }

    pub async fn update_admin(
        &self,
        id: &str,
        request: UpdateAdminRequest,
    ) -> Result<AdminUserResponse, ServiceError> {
        // 检查用户是否存在
        let existing_user = self
            .find_by_id(id)
            .await?
            .ok_or(ServiceError::UserNotFound)?;

        // 如果要更新用户名，检查是否与其他用户冲突
        if let Some(ref new_username) = request.username {
            if new_username != &existing_user.username {
                if self.username_exists(new_username).await? {
                    return Err(ServiceError::UserAlreadyExists);
                }
            }
        }

        // 准备更新字段
        let username = request
            .username
            .as_deref()
            .unwrap_or(&existing_user.username);
        let is_super_admin = request
            .is_super_admin
            .unwrap_or(existing_user.is_super_admin);

        // 处理密码更新
        let password = if let Some(new_password) = request.password {
            bcrypt::hash(&new_password, self.bcrypt_cost)?
        } else {
            existing_user.password.clone()
        };

        // 执行更新
        sqlx::query(
            r#"
            UPDATE admin_users 
            SET username = ?, password = ?, is_super_admin = ?, updated_at = CURRENT_TIMESTAMP
            WHERE id = ?
            "#,
        )
        .bind(username)
        .bind(&password)
        .bind(is_super_admin)
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(AdminUserResponse {
            id: id.to_string(),
            username: username.to_string(),
            is_super_admin,
        })
    }

    pub async fn delete_admin(&self, id: &str) -> Result<(), ServiceError> {
        // 检查用户是否存在
        let user = self
            .find_by_id(id)
            .await?
            .ok_or(ServiceError::UserNotFound)?;

        // 防止删除超级管理员（可选的业务规则）
        if user.is_super_admin {
            // 检查是否是最后一个超级管理员
            let super_admin_count = self.count_super_admins().await?;
            if super_admin_count <= 1 {
                return Err(ServiceError::CannotDeleteSuperAdmin);
            }
        }

        // 执行删除
        let result = sqlx::query("DELETE FROM admin_users WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(ServiceError::UserNotFound);
        }

        Ok(())
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<AdminUser>, sqlx::Error> {
        sqlx::query_as::<_, AdminUser>(
            r#"
            SELECT id, username, password, is_super_admin, created_at, updated_at 
            FROM admin_users 
            WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    async fn username_exists(&self, username: &str) -> Result<bool, sqlx::Error> {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM admin_users WHERE username = ?")
            .bind(username)
            .fetch_one(&self.pool)
            .await?;

        Ok(count.0 > 0)
    }

    async fn count_super_admins(&self) -> Result<i64, sqlx::Error> {
        let count: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM admin_users WHERE is_super_admin = TRUE")
                .fetch_one(&self.pool)
                .await?;

        Ok(count.0)
    }
}
