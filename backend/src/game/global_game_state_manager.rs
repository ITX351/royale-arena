//! 全局游戏状态管理器
//! 负责管理所有游戏的内存状态，与REST API服务分离

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde_json::Value as JsonValue;
use sqlx::MySqlPool;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::game::models::{GameStatus, SaveFileInfo};
use crate::websocket::models::{GameState, Place, Player};

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

    /// 获取游戏状态（如果不存在则返回错误）
    pub async fn get_game_state(&self, game_id: &str) -> Result<Arc<RwLock<GameState>>, String> {
        // 检查内存中是否已存在游戏状态
        if let Some(game_state) = self.game_states.get(game_id) {
            return Ok(game_state.clone());
        }

        // 如果内存中不存在游戏状态，返回错误
        Err("Game state not found in memory".to_string())
    }

    /// 创建新的游戏状态（仅由首次开始游戏时调用）
    pub async fn create_game_state(
        &self,
        game_id: &str,
        rules_config: JsonValue,
    ) -> Result<Arc<RwLock<GameState>>, String> {
        // 检查内存中是否已存在游戏状态
        if self.game_states.contains_key(game_id) {
            return Err("Game state already exists".to_string());
        }

        // 创建新的游戏状态
        let mut game_state = GameState::new(game_id.to_string(), rules_config);

        // 从数据库加载玩家信息
        let players_result = sqlx::query!(
            "SELECT id, name, password, team_id FROM actors WHERE game_id = ?",
            game_id
        )
        .fetch_all(&self.pool)
        .await;

        if let Ok(players) = players_result {
            for player_record in players {
                let player = Player::new(
                    player_record.id,
                    player_record.name,
                    player_record.password,
                    player_record.team_id as u32,
                    &game_state.rule_engine, // 传递规则引擎引用
                );
                game_state.players.insert(player.id.clone(), player);
            }
        }

        // 从游戏规则引擎的MapConfig中加载地点信息，而不是从JSON中解析
        for place_name in &game_state.rule_engine.map_config.places {
            let place = Place::new(place_name.clone());
            game_state.places.insert(place_name.clone(), place);
        }

        // 将新创建的游戏状态存储到内存中
        let game_state_arc = Arc::new(RwLock::new(game_state));
        self.game_states
            .insert(game_id.to_string(), game_state_arc.clone());

        Ok(game_state_arc)
    }

    /// 保存游戏状态到磁盘（使用时间戳生成文件名）
    pub async fn save_game_state_to_disk(&self, game_id: &str) -> Result<String, String> {
        // 生成带时间戳的文件名（使用Windows兼容格式）
        let timestamp = Utc::now().format("%Y-%m-%dT%H-%M-%S%.3fZ").to_string();
        let file_name = format!("{}.json", timestamp);

        self.save_game_state_to_disk_with_name(game_id, &file_name)
            .await?;
        Ok(file_name)
    }

    /// 保存游戏状态到磁盘（指定文件名）
    pub async fn save_game_state_to_disk_with_name(
        &self,
        game_id: &str,
        file_name: &str,
    ) -> Result<(), String> {
        if let Some(game_state) = self.game_states.get(game_id) {
            // 在代码块中获取锁、克隆数据并更新save_time
            let cloned_game_state = {
                let game_state_guard = game_state.read().await;
                // 克隆游戏状态
                let mut cloned = (*game_state_guard).clone();
                // 更新克隆的游戏状态的保存时间
                cloned.save_time = Some(Utc::now());
                // 代码块结束时自动释放锁
                cloned
            };

            let serialized = serde_json::to_string(&cloned_game_state)
                .map_err(|e| format!("Failed to serialize game state: {}", e))?;

            let file_path = format!("game_states/{}/{}", game_id, file_name);
            if let Some(parent) = Path::new(&file_path).parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create directory: {}", e))?;
            }

            tracing::debug!("Saving game state to disk: {}", file_path);
            fs::write(&file_path, serialized)
                .map_err(|e| format!("Failed to write game state to disk: {}", e))?;

            Ok(())
        } else {
            Err("Game state not found".to_string())
        }
    }

    /// 从磁盘恢复游戏状态（指定文件名）
    pub async fn load_game_state_from_disk_with_name(
        &self,
        game_id: &str,
        file_name: &str,
    ) -> Result<(), String> {
        let file_path = format!("game_states/{}/{}", game_id, file_name);
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

    /// 获取指定游戏的所有存档文件列表
    pub async fn list_save_files(&self, game_id: &str) -> Result<Vec<SaveFileInfo>, String> {
        let dir_path = format!("game_states/{}", game_id);
        if !Path::new(&dir_path).exists() {
            return Ok(vec![]); // 如果目录不存在，返回空列表
        }

        let mut save_files = Vec::new();

        // 读取目录中的所有文件
        let entries =
            fs::read_dir(&dir_path).map_err(|e| format!("Failed to read directory: {}", e))?;

        for entry in entries {
            let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
            let path = entry.path();

            // 只处理.json文件
            if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
                if let Some(file_name) = path.file_name().and_then(|name| name.to_str()) {
                    // 尝试从文件名解析时间戳
                    // 文件名格式: 2023-01-01T10-00-00.000Z.json (Windows兼容格式)
                    let created_at = if let Some(timestamp_str) = file_name.strip_suffix(".json") {
                        // 将Windows兼容格式转换回ISO 8601格式进行解析
                        // 将 "T" 后面的 "-" 替换为 ":" 来构造有效的 RFC3339 时间字符串
                        if let Some(t_pos) = timestamp_str.find('T') {
                            let (date_part, time_part) = timestamp_str.split_at(t_pos + 1);
                            let time_part = time_part.replace("-", ":");
                            let iso_timestamp = format!("{}{}", date_part, time_part);
                            DateTime::parse_from_rfc3339(&iso_timestamp)
                                .map(|dt| Some(dt.with_timezone(&Utc)))
                                .unwrap_or(None) // 如果解析失败，使用None
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    save_files.push(SaveFileInfo {
                        file_name: file_name.to_string(),
                        created_at,
                    });
                }
            }
        }

        // 按创建时间排序，最新的在前
        save_files.sort_by(|a, b| {
            // 处理None值的情况
            match (&a.created_at, &b.created_at) {
                (Some(a_time), Some(b_time)) => b_time.cmp(a_time),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => std::cmp::Ordering::Equal,
            }
        });

        Ok(save_files)
    }

    /// 移除内存中的游戏状态
    pub fn remove_game_state(&self, game_id: &str) -> bool {
        self.game_states.remove(game_id).is_some()
    }

    /// 检查游戏是否接受连接
    pub async fn is_status_accepting_connections(status: &GameStatus) -> bool {
        matches!(status, GameStatus::Running)
    }
}
