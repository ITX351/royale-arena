//! 全局游戏状态管理器
//! 负责管理所有游戏的内存状态，与REST API服务分离

use std::sync::Arc;
use tokio::sync::RwLock;
use serde_json::Value as JsonValue;
use sqlx::MySqlPool;
use std::fs;
use std::path::Path;
use dashmap::DashMap;

// 从websocket模块导入游戏模型
use crate::websocket::models::{GameState, Player, Place};
use crate::game::models::GameStatus;

/// 全局游戏状态管理器
#[derive(Clone)]
pub struct GlobalGameStateManager {
    /// 数据库连接池
    pool: MySqlPool,
    /// 游戏状态存储（内存中）
    game_states: Arc<DashMap<String, Arc<RwLock<GameState>>>>,
}

impl GlobalGameStateManager {
    /// 创建新的全局游戏状态管理器
    pub fn new(pool: MySqlPool) -> Self {
        Self {
            pool,
            game_states: Arc::new(DashMap::new()),
        }
    }

    /// 获取游戏状态（如果不存在则创建）
    pub async fn get_game_state(&self, game_id: &str, rules_config: JsonValue) -> Result<Arc<RwLock<GameState>>, String> {
        // 检查内存中是否已存在游戏状态
        if let Some(game_state) = self.game_states.get(game_id) {
            return Ok(game_state.clone());
        }

        // 从磁盘加载或创建新的游戏状态
        match self.load_game_state_from_disk(game_id).await {
            Ok(()) => {
                // 加载成功，返回内存中的状态
                if let Some(game_state) = self.game_states.get(game_id) {
                    return Ok(game_state.clone());
                }
            }
            Err(_) => {
                // 加载失败，创建新的游戏状态
            }
        }

        // 创建新的游戏状态
        let mut game_state = GameState::new(game_id.to_string(), rules_config.clone());

        // 从数据库加载玩家信息
        let players_result = sqlx::query!(
            "SELECT id, name, team_id FROM actors WHERE game_id = ?",
            game_id
        )
        .fetch_all(&self.pool)
        .await;

        if let Ok(players) = players_result {
            for player_record in players {
                let player = Player::new(
                    player_record.id,
                    player_record.name,
                    player_record.team_id as u32,
                );
                game_state.players.insert(player.id.clone(), player);
            }
        }

        // 从游戏规则配置中加载地点信息，而不是从不存在的places表中查询
        if let Some(map_config) = rules_config.get("map") {
            if let Some(places_config) = map_config.get("places").and_then(|p| p.as_array()) {
                for place_name in places_config {
                    if let Some(name) = place_name.as_str() {
                        let place = Place::new(name.to_string());
                        game_state.places.insert(name.to_string(), place);
                    }
                }
            }
        }

        // 将新创建的游戏状态存储到内存中
        let game_state_arc = Arc::new(RwLock::new(game_state));
        self.game_states.insert(game_id.to_string(), game_state_arc.clone());
        
        Ok(game_state_arc)
    }

    /// 保存游戏状态到磁盘
    pub async fn save_game_state_to_disk(&self, game_id: &str) -> Result<(), String> {
        if let Some(game_state) = self.game_states.get(game_id) {
            let game_state_guard = game_state.read().await;
            let serialized = serde_json::to_string(&*game_state_guard)
                .map_err(|e| format!("Failed to serialize game state: {}", e))?;

            let file_path = format!("game_states/{}.json", game_id);
            if let Some(parent) = Path::new(&file_path).parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create directory: {}", e))?;
            }

            fs::write(&file_path, serialized)
                .map_err(|e| format!("Failed to write game state to disk: {}", e))?;

            Ok(())
        } else {
            Err("Game state not found".to_string())
        }
    }

    /// 从磁盘恢复游戏状态
    pub async fn load_game_state_from_disk(&self, game_id: &str) -> Result<(), String> {
        let file_path = format!("game_states/{}.json", game_id);
        if !Path::new(&file_path).exists() {
            return Err("Game state file not found".to_string());
        }

        let serialized = fs::read_to_string(&file_path)
            .map_err(|e| format!("Failed to read game state from disk: {}", e))?;

        let game_state: GameState = serde_json::from_str(&serialized)
            .map_err(|e| format!("Failed to deserialize game state: {}", e))?;

        let game_state_arc = Arc::new(RwLock::new(game_state));
        self.game_states.insert(game_id.to_string(), game_state_arc);

        Ok(())
    }
    /// 检查游戏是否接受连接
    pub async fn is_status_accepting_connections(status: &GameStatus) -> bool {
        match status {
            GameStatus::Running | GameStatus::Paused => true,
            _ => false,
        }
    }
}