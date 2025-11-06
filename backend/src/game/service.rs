use sqlx::MySqlPool;

use super::errors::GameError;
use super::models::*;
use crate::rule_template::models::RuleTemplate;

#[derive(Clone)]
pub struct GameService {
    pool: MySqlPool,
}

impl GameService {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    /// 创建新游戏
    pub async fn create_game(&self, request: CreateGameRequest) -> Result<Game, GameError> {
        // 验证请求参数
        request.validate().map_err(GameError::ValidationError)?;

        let CreateGameRequest {
            id,
            name,
            description,
            director_password,
            max_players,
            rule_template_id,
        } = request;

        let normalized_id = id.trim().to_string();

        let existing_id = sqlx::query("SELECT id FROM games WHERE id = ?")
            .bind(&normalized_id)
            .fetch_optional(&self.pool)
            .await?;

        if existing_id.is_some() {
            return Err(GameError::GameIdExists);
        }

        // 检查游戏名称是否已存在
        let existing_game = sqlx::query!("SELECT id FROM games WHERE name = ?", &name)
            .fetch_optional(&self.pool)
            .await?;

        if existing_game.is_some() {
            return Err(GameError::GameNameExists);
        }

        // 从规则模板获取配置
        let template = self.get_rule_template(&rule_template_id).await?;
        let rules_config = template.rules_config;

        let game_id_for_insert = normalized_id.clone();
        let game_id_for_fetch = normalized_id.clone();

        // 插入新游戏记录
        sqlx::query!(
            r#"
            INSERT INTO games (id, name, description, director_password, max_players, status, rules_config)
            VALUES (?, ?, ?, ?, ?, 'waiting', ?)
            "#,
            game_id_for_insert,
            name,
            description,
            director_password,
            max_players,
            &rules_config
        )
        .execute(&self.pool)
        .await?;

        // 查询并返回创建的游戏
        let game = sqlx::query_as!(
            Game,
            r#"
            SELECT id, name, description, director_password, max_players, 
                   status as "status: GameStatus", rules_config, created_at, updated_at
            FROM games 
            WHERE id = ?
            "#,
            game_id_for_fetch
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(game)
    }

    /// 更新游戏信息
    pub async fn update_game(
        &self,
        game_id: &str,
        request: UpdateGameRequest,
    ) -> Result<Game, GameError> {
        // 验证请求参数
        request.validate().map_err(GameError::ValidationError)?;

        // 检查游戏是否存在
        let existing_game = sqlx::query!("SELECT id FROM games WHERE id = ?", game_id)
            .fetch_optional(&self.pool)
            .await?;

        if existing_game.is_none() {
            return Err(GameError::GameNotFound);
        }

        // 如果要更新游戏名称，检查新名称是否已被其他游戏使用
        if let Some(ref new_name) = request.name {
            let name_conflict = sqlx::query!(
                "SELECT id FROM games WHERE name = ? AND id != ?",
                new_name,
                game_id
            )
            .fetch_optional(&self.pool)
            .await?;

            if name_conflict.is_some() {
                return Err(GameError::GameNameExists);
            }
        }

        // 检查是否有要更新的字段
        if request.name.is_none()
            && request.description.is_none()
            && request.director_password.is_none()
            && request.max_players.is_none()
            && request.rules_config.is_none()
        {
            // 如果没有要更新的字段，直接返回当前游戏信息
            return self.get_game_by_id(game_id).await;
        }

        // 执行字段更新操作
        if let Some(ref name) = request.name {
            sqlx::query!(
                "UPDATE games SET name = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
                name,
                game_id
            )
            .execute(&self.pool)
            .await?;
        }
        if let Some(ref description) = request.description {
            sqlx::query!(
                "UPDATE games SET description = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
                description,
                game_id
            )
            .execute(&self.pool)
            .await?;
        }
        if let Some(ref password) = request.director_password {
            sqlx::query!("UPDATE games SET director_password = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?", password, game_id)
                .execute(&self.pool).await?;
        }
        if let Some(max_players) = request.max_players {
            sqlx::query!(
                "UPDATE games SET max_players = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
                max_players,
                game_id
            )
            .execute(&self.pool)
            .await?;
        }
        // 注意：我们不再更新 rule_template_id，因为已经移除了这个字段
        // 如果需要更新规则配置，可以在这里处理
        if let Some(ref rules_config) = request.rules_config {
            sqlx::query!(
                "UPDATE games SET rules_config = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
                rules_config,
                game_id
            )
            .execute(&self.pool)
            .await?;
        }

        // 查询并返回更新后的游戏
        self.get_game_by_id(game_id).await
    }

    /// 删除游戏
    pub async fn delete_game(&self, game_id: &str) -> Result<(), GameError> {
        let result = sqlx::query!("DELETE FROM games WHERE id = ?", game_id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(GameError::GameNotFound);
        }

        Ok(())
    }

    /// 获取游戏列表（支持筛选）
    pub async fn get_games(
        &self,
        query: &GameListQuery,
        include_director_password: bool,
    ) -> Result<Vec<GameListItem>, GameError> {
        let base_query = r#"
            SELECT g.id, g.name, g.description, g.status, g.max_players, g.created_at,
                   COUNT(a.id) as player_count,
                   g.director_password
            FROM games g
            LEFT JOIN actors a ON g.id = a.game_id
        "#;

        let results = if let Some(ref filter) = query.filter {
            // 根据筛选类型获取对应的状态列表
            let status_list = filter.get_status_list();

            // 构建 WHERE 子句
            let status_placeholders = status_list
                .iter()
                .map(|_| "?")
                .collect::<Vec<_>>()
                .join(", ");
            let full_query = format!(
                "{} WHERE g.status IN ({}) GROUP BY g.id ORDER BY g.created_at DESC",
                base_query, status_placeholders
            );

            // 执行查询
            let mut query_builder = sqlx::query_as::<_, GameQueryResult>(&full_query);
            for status in status_list {
                query_builder = query_builder.bind(status);
            }

            query_builder.fetch_all(&self.pool).await?
        } else {
            // 默认显示所有非隐藏和非删除的游戏（相当于"All"筛选）
            let full_query = format!(
                "{} WHERE g.status NOT IN ('hidden', 'deleted') GROUP BY g.id ORDER BY g.created_at DESC",
                base_query
            );
            sqlx::query_as::<_, GameQueryResult>(&full_query)
                .fetch_all(&self.pool)
                .await?
        };

        let game_list = results
            .into_iter()
            .map(|mut result| {
                if !include_director_password {
                    result.director_password = None;
                }
                GameListItem::from(result)
            })
            .collect();

        Ok(game_list)
    }

    /// 获取游戏详情（包含规则信息）
    pub async fn get_game_by_id_with_player_counts(
        &self,
        game_id: &str,
        include_director_password: bool,
    ) -> Result<GameWithPlayerCounts, GameError> {
        // 直接查询游戏表，无需JOIN规则模板表
        let game_info = sqlx::query!(
            r#"
            SELECT id, name, description, status as "status: GameStatus", 
                   max_players, created_at, director_password, rules_config
            FROM games
            WHERE id = ?
            "#,
            game_id
        )
        .fetch_optional(&self.pool)
        .await?;

        let game_info = game_info.ok_or(GameError::GameNotFound)?;

        // 获取玩家数量
        let player_count = self.get_player_count(game_id).await?;

        // 构建响应
        Ok(GameWithPlayerCounts {
            id: game_info.id,
            name: game_info.name,
            description: game_info.description,
            status: game_info.status,
            player_count,
            max_players: game_info.max_players,
            created_at: game_info.created_at,
            director_password: if include_director_password {
                Some(game_info.director_password)
            } else {
                None
            },
            rules_config: game_info.rules_config,
        })
    }

    /// 验证游戏身份
    pub async fn authenticate_game(
        &self,
        game_id: &str,
        password: &str,
    ) -> Result<GameAuthenticationResponse, GameError> {
        let actor = sqlx::query!(
            "SELECT id, name FROM actors WHERE game_id = ? AND password = ?",
            game_id,
            password
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(actor) = actor {
            return Ok(GameAuthenticationResponse {
                role: GameAuthenticationRole::Actor,
                actor_id: Some(actor.id),
                actor_name: Some(actor.name),
            });
        }

        let game = sqlx::query!("SELECT director_password FROM games WHERE id = ?", game_id)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(game) = game {
            if game.director_password == password {
                return Ok(GameAuthenticationResponse {
                    role: GameAuthenticationRole::Director,
                    actor_id: None,
                    actor_name: None,
                });
            }
        }

        Ok(GameAuthenticationResponse {
            role: GameAuthenticationRole::Invalid,
            actor_id: None,
            actor_name: None,
        })
    }

    /// 根据ID获取游戏信息
    pub async fn get_game_by_id(&self, game_id: &str) -> Result<Game, GameError> {
        let game = sqlx::query_as!(
            Game,
            r#"
            SELECT id, name, description, director_password, max_players, 
                   status as "status: GameStatus", rules_config, created_at, updated_at
            FROM games 
            WHERE id = ?
            "#,
            game_id
        )
        .fetch_optional(&self.pool)
        .await?;

        game.ok_or(GameError::GameNotFound)
    }

    /// 获取游戏的玩家数量
    async fn get_player_count(&self, game_id: &str) -> Result<i32, GameError> {
        let count = sqlx::query!(
            "SELECT COUNT(*) as count FROM actors WHERE game_id = ?",
            game_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(count.count as i32)
    }

    /// 新增辅助方法：获取规则模板
    async fn get_rule_template(&self, template_id: &str) -> Result<RuleTemplate, GameError> {
        let template = sqlx::query!(
            "SELECT id, template_name, description, rules_config FROM rule_templates WHERE id = ? AND is_active = true",
            template_id
        )
        .fetch_optional(&self.pool)
        .await?;

        template
            .ok_or(GameError::RuleTemplateNotFound)
            .map(|t| RuleTemplate {
                id: t.id,
                template_name: t.template_name,
                description: t.description,
                is_active: true, // 查询中已经限制了is_active = true
                rules_config: t.rules_config,
                created_at: chrono::Utc::now(), // 这些字段在当前上下文中不重要
                updated_at: chrono::Utc::now(),
            })
    }
}
