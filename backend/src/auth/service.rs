use crate::admin::models::{AdminUser, JwtClaims, LoginRequest, LoginResponse};
use crate::auth::jwt::JwtManager;
use crate::database::DatabasePool;
use crate::errors::{AuthError, ServiceError};

#[derive(Clone)]
pub struct AuthService {
    pool: DatabasePool,
    jwt_manager: JwtManager,
}

impl AuthService {
    pub fn new(pool: DatabasePool, jwt_manager: JwtManager) -> Self {
        Self { pool, jwt_manager }
    }

    pub async fn login(&self, credentials: LoginRequest) -> Result<LoginResponse, ServiceError> {
        // 查找用户
        let user = self.find_by_username(&credentials.username).await?
            .ok_or(AuthError::InvalidCredentials)?;

        // 验证密码
        if !bcrypt::verify(&credentials.password, &user.password)? {
            return Err(AuthError::InvalidCredentials.into());
        }

        // 生成 JWT token
        let token = self.jwt_manager.generate_token(
            &user.id,
            &user.username,
            user.is_super_admin,
        )?;

        Ok(LoginResponse {
            success: true,
            token,
            user: user.into(), // 转换为不包含密码的响应结构
            expires_in: self.jwt_manager.expiration_hours * 3600, // 转换为秒
        })
    }

    pub async fn validate_token(&self, token: &str) -> Result<JwtClaims, ServiceError> {
        let claims = self.jwt_manager.validate_token(token)?;
        
        // 验证用户是否仍然存在
        let user = self.find_by_id(&claims.sub).await?
            .ok_or(AuthError::UserNotFound)?;

        // 确保claims中的信息是最新的
        if user.username != claims.username || user.is_super_admin != claims.is_super_admin {
            return Err(AuthError::InvalidToken.into());
        }

        Ok(claims)
    }

    async fn find_by_username(&self, username: &str) -> Result<Option<AdminUser>, sqlx::Error> {
        sqlx::query_as::<_, AdminUser>(
            r#"
            SELECT id, username, password, is_super_admin, created_at, updated_at 
            FROM admin_users 
            WHERE username = ?
            "#
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<AdminUser>, sqlx::Error> {
        sqlx::query_as::<_, AdminUser>(
            r#"
            SELECT id, username, password, is_super_admin, created_at, updated_at 
            FROM admin_users 
            WHERE id = ?
            "#
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }
}