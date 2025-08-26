use sqlx::MySqlPool;
use uuid::Uuid;

use super::errors::GameError;
use super::models::*;

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
        
        // 检查游戏名称是否已存在
        let existing_game = sqlx::query!(
            "SELECT id FROM games WHERE name = ?",
            request.name
        )
        .fetch_optional(&self.pool)
        .await?;
        
        if existing_game.is_some() {
            return Err(GameError::GameNameExists);
        }
        
        // 如果提供了规则模板ID，验证其存在性
        if let Some(ref template_id) = request.rule_template_id {
            self.validate_rule_template_exists(template_id).await?;
        }
        
        // 生成新的游戏ID
        let game_id = Uuid::new_v4().to_string();
        
        // 插入新游戏记录
        sqlx::query!(
            r#"
            INSERT INTO games (id, name, description, director_password, max_players, status, rule_template_id)
            VALUES (?, ?, ?, ?, ?, 'waiting', ?)
            "#,
            game_id,
            request.name,
            request.description,
            request.director_password,
            request.max_players,
            request.rule_template_id
        )
        .execute(&self.pool)
        .await?;
        
        // 查询并返回创建的游戏
        let game = sqlx::query_as!(
            Game,
            r#"
            SELECT id, name, description, director_password, max_players, 
                   status as "status: GameStatus", rule_template_id, created_at, updated_at
            FROM games 
            WHERE id = ?
            "#,
            game_id
        )
        .fetch_one(&self.pool)
        .await?;
        
        Ok(game)
    }

    /// 更新游戏信息
    pub async fn update_game(&self, game_id: &str, request: UpdateGameRequest) -> Result<Game, GameError> {
        // 验证请求参数
        request.validate().map_err(GameError::ValidationError)?;
        
        // 检查游戏是否存在
        let existing_game = sqlx::query!(
            "SELECT id FROM games WHERE id = ?",
            game_id
        )
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
        
        // 如果提供了新的规则模板ID，验证其存在性
        if let Some(ref template_id) = request.rule_template_id {
            self.validate_rule_template_exists(template_id).await?;
        }
        
        // 检查是否有要更新的字段
        if request.name.is_none() && request.description.is_none() && 
           request.director_password.is_none() && request.max_players.is_none() && 
           request.rule_template_id.is_none() {
            // 如果没有要更新的字段，直接返回当前游戏信息
            return self.get_game_by_id(game_id).await;
        }
        // 执行字段更新操作
        if let Some(ref name) = request.name {
            sqlx::query!("UPDATE games SET name = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?", name, game_id)
                .execute(&self.pool).await?;
        }
        if let Some(ref description) = request.description {
            sqlx::query!("UPDATE games SET description = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?", description, game_id)
                .execute(&self.pool).await?;
        }
        if let Some(ref password) = request.director_password {
            sqlx::query!("UPDATE games SET director_password = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?", password, game_id)
                .execute(&self.pool).await?;
        }
        if let Some(max_players) = request.max_players {
            sqlx::query!("UPDATE games SET max_players = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?", max_players, game_id)
                .execute(&self.pool).await?;
        }
        if let Some(ref template_id) = request.rule_template_id {
            sqlx::query!("UPDATE games SET rule_template_id = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?", template_id, game_id)
                .execute(&self.pool).await?;
        }
        
        // 查询并返回更新后的游戏
        self.get_game_by_id(game_id).await
    }

    /// 删除游戏
    pub async fn delete_game(&self, game_id: &str) -> Result<(), GameError> {
        let result = sqlx::query!(
            "DELETE FROM games WHERE id = ?",
            game_id
        )
        .execute(&self.pool)
        .await?;
        
        if result.rows_affected() == 0 {
            return Err(GameError::GameNotFound);
        }
        
        Ok(())
    }

    /// 获取游戏列表（支持筛选）
    pub async fn get_games(&self, query: &GameListQuery) -> Result<Vec<GameListItem>, GameError> {
        let base_query = r#"
            SELECT g.id, g.name, g.description, g.status, g.max_players, g.created_at,
                   COUNT(a.id) as player_count
            FROM games g
            LEFT JOIN actors a ON g.id = a.game_id
        "#;
        
        let results = if let Some(ref filter) = query.filter {
            // 根据筛选类型获取对应的状态列表
            let status_list = filter.get_status_list();
            
            // 构建 WHERE 子句
            let status_placeholders = status_list.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
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
            .map(GameListItem::from)
            .collect();
        
        Ok(game_list)
    }

    /// 获取游戏详情（包含规则信息）
    pub async fn get_game_with_rules(&self, game_id: &str) -> Result<GameWithRules, GameError> {
        // 获取游戏基本信息和规则模板信息
        let game_info = sqlx::query!(
            r#"
            SELECT g.id, g.name, g.description, g.status as "status: GameStatus", g.max_players, g.created_at,
                   rt.id as rule_template_id, rt.template_name, rt.description as rule_description, rt.rules_config
            FROM games g
            LEFT JOIN rule_templates rt ON g.rule_template_id = rt.id
            WHERE g.id = ?
            "#,
            game_id
        )
        .fetch_optional(&self.pool)
        .await?;
        
        let game_info = game_info.ok_or(GameError::GameNotFound)?;
        
        // 获取玩家数量
        let player_count = self.get_player_count(game_id).await?;
        
        // 构建规则模板信息
        let rule_template = if let Some(template_id) = game_info.rule_template_id {
            Some(RuleTemplateInfo {
                id: template_id,
                template_name: game_info.template_name.unwrap_or_default(),
                description: game_info.rule_description,
                rules_config: game_info.rules_config.unwrap_or(serde_json::Value::Null),
            })
        } else {
            None
        };
        
        Ok(GameWithRules {
            id: game_info.id,
            name: game_info.name,
            description: game_info.description,
            status: game_info.status,
            player_count,
            max_players: game_info.max_players,
            created_at: game_info.created_at,
            rule_template,
        })
    }

    /// 根据ID获取游戏信息
    async fn get_game_by_id(&self, game_id: &str) -> Result<Game, GameError> {
        let game = sqlx::query_as!(
            Game,
            r#"
            SELECT id, name, description, director_password, max_players, 
                   status as "status: GameStatus", rule_template_id, created_at, updated_at
            FROM games 
            WHERE id = ?
            "#,
            game_id
        )
        .fetch_optional(&self.pool)
        .await?;
        
        game.ok_or(GameError::GameNotFound)
    }

    /// 验证规则模板是否存在
    async fn validate_rule_template_exists(&self, template_id: &str) -> Result<(), GameError> {
        let exists = sqlx::query!(
            "SELECT id FROM rule_templates WHERE id = ? AND is_active = TRUE",
            template_id
        )
        .fetch_optional(&self.pool)
        .await?;
        
        if exists.is_none() {
            return Err(GameError::RuleTemplateNotFound);
        }
        
        Ok(())
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
}