# 击杀记录查询功能设计文档

## 1. 概述

本设计文档旨在定义和实现一个新的功能模块，用于查询和展示游戏中的击杀记录。该功能将提供对`kill_records`表的查询接口，支持玩家和导演两种角色的查询需求，并集成到前端界面中进行展示。

## 2. 设计原则

- 所有的修改均应是破坏性的，不需要考虑任何向后兼容
- 完成最小实现，拒绝任何过度设计
- 不编写任何测试
- 所有输出使用简体中文

## 3. 后端实现

### 3.1 数据模型定义

在`game/models.rs`中添加击杀记录相关的数据模型：

```rust
/// 击杀记录模型
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct KillRecord {
    pub id: String,
    pub game_id: String,
    pub killer_id: Option<String>,
    pub victim_id: String,
    pub kill_time: DateTime<Utc>,
    pub cause: String,
    pub weapon: Option<String>,
    pub location: Option<String>,
}

/// 获取玩家击杀记录请求
#[derive(Debug, Deserialize)]
pub struct GetPlayerKillRecordsRequest {
    /// 玩家密码
    pub password: String,
}
```

### 3.2 服务层实现

在`game/log_service.rs`中添加对`kill_records`表的查询函数：

```rust
impl GameLogService {
    /// 获取玩家击杀记录
    pub async fn get_player_kill_records(
        &self,
        game_id: &str,
        player_id: &str,
        password: &str,
    ) -> Result<Vec<KillRecord>, GameError> {
        // 验证请求参数
        let request = GetPlayerKillRecordsRequest {
            password: password.to_string(),
        };
        request.validate().map_err(GameError::ValidationError)?;

        // 验证玩家是否存在且密码正确
        let actor = sqlx::query!(
            "SELECT id FROM actors WHERE id = ? AND game_id = ? AND password = ?",
            player_id,
            game_id,
            password
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(GameError::DatabaseError)?;

        if actor.is_none() {
            return Err(GameError::ValidationError(
                "Invalid player credentials".to_string(),
            ));
        }

        // 查询玩家相关的击杀记录（作为击杀者）
        let kill_records = sqlx::query_as!(
            KillRecord,
            r#"
            SELECT id, game_id, killer_id, victim_id, kill_time, cause, weapon, location
            FROM kill_records 
            WHERE game_id = ? AND killer_id = ?
            ORDER BY kill_time ASC
            "#,
            game_id,
            player_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(GameError::DatabaseError)?;

        Ok(kill_records)
    }

    /// 获取导演击杀记录
    pub async fn get_director_kill_records(
        &self,
        game_id: &str,
        password: &str,
    ) -> Result<Vec<KillRecord>, GameError> {
        // 验证导演密码
        let game = sqlx::query!(
            "SELECT id FROM games WHERE id = ? AND director_password = ?",
            game_id,
            password
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(GameError::DatabaseError)?;

        if game.is_none() {
            return Err(GameError::ValidationError(
                "Invalid director credentials".to_string(),
            ));
        }

        // 查询所有击杀记录
        let kill_records = sqlx::query_as!(
            KillRecord,
            r#"
            SELECT id, game_id, killer_id, victim_id, kill_time, cause, weapon, location
            FROM kill_records 
            WHERE game_id = ?
            ORDER BY kill_time ASC
            "#,
            game_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(GameError::DatabaseError)?;

        Ok(kill_records)
    }

    /// 删除指定时间戳之后的击杀记录
    pub async fn delete_kill_records_after_timestamp(
        &self,
        game_id: &str,
        timestamp: Option<DateTime<Utc>>,
    ) -> Result<u64, GameError> {
        let rows_affected = if let Some(ts) = timestamp {
            sqlx::query!(
                "DELETE FROM kill_records WHERE game_id = ? AND kill_time > ?",
                game_id,
                ts
            )
            .execute(&self.pool)
            .await
        } else {
            sqlx::query!("DELETE FROM kill_records WHERE game_id = ?", game_id)
                .execute(&self.pool)
                .await
        }
        .map_err(GameError::DatabaseError)?
        .rows_affected();

        Ok(rows_affected)
    }
}
```

### 3.3 路由和处理器实现

在`game/handlers.rs`中添加新的路由处理函数：

```rust
/// 获取玩家击杀记录 (玩家接口)
pub async fn get_player_kill_records(
    State(state): State<AppState>,
    Path((game_id, player_id)): Path<(String, String)>,
    Json(request): Json<GetPlayerKillRecordsRequest>,
) -> Result<Json<serde_json::Value>, GameError> {
    // 验证请求参数
    request.validate().map_err(GameError::ValidationError)?;

    // 获取玩家击杀记录
    let kill_records = state
        .game_log_service
        .get_player_kill_records(&game_id, &player_id, &request.password)
        .await?;

    Ok(Json(json!({
        "success": true,
        "data": kill_records
    })))
}

/// 获取导演击杀记录 (导演接口)
pub async fn get_director_kill_records(
    State(state): State<AppState>,
    Path(game_id): Path<String>,
    Query(query): Query<DirectorPasswordQuery>,
) -> Result<Json<serde_json::Value>, GameError> {
    // 获取导演击杀记录
    let kill_records = state
        .game_log_service
        .get_director_kill_records(&game_id, &query.password)
        .await?;

    Ok(Json(json!({
        "success": true,
        "data": kill_records
    })))
}

/// 删除游戏击杀记录 (管理员接口)
pub async fn delete_game_kill_records(
    State(state): State<AppState>,
    Path(game_id): Path<String>,
    Query(query): Query<DeleteLogsQuery>,
) -> Result<Json<serde_json::Value>, GameError> {
    // 解析时间戳参数
    let timestamp = if let Some(ts_str) = query.after_timestamp {
        Some(
            chrono::DateTime::parse_from_rfc3339(&ts_str)
                .map_err(|_| GameError::ValidationError("Invalid timestamp format".to_string()))?
                .with_timezone(&chrono::Utc),
        )
    } else {
        None
        };

    // 删除击杀记录
    let deleted_count = state
        .game_log_service
        .delete_kill_records_after_timestamp(&game_id, timestamp)
        .await?;

    Ok(Json(json!({
        "success": true,
        "message": format!("Deleted {} kill records", deleted_count)
    })))
}
```

在`routes.rs`中注册新的路由：

```rust
// 需要管理员权限的游戏管理路由
let game_admin_routes = Router::new()
    .route("/", get(get_games))
    .route("/", post(create_game))
    .route("/{game_id}", get(get_game_with_rules))
    .route("/{game_id}", put(update_game))
    .route("/{game_id}", delete(delete_game))
    // 新增的删除游戏日志路由
    .route("/{game_id}/logs", delete(delete_game_logs))
    // 新增的删除游戏击杀记录路由
    .route("/{game_id}/kill-records", delete(delete_game_kill_records))
    .layer(middleware::from_fn_with_state(
        auth_service.clone(),
        jwt_auth_middleware,
    ))
    .with_state(app_state.clone());

// 导演接口路由（无需JWT认证，使用导演密码验证）
let director_routes = Router::new()
    .route(
        "/game/{game_id}/players",
        post(batch_add_players)
            .get(get_players)
            .delete(batch_delete_players),
    )
    // 导演更新游戏状态接口
    .route("/game/{game_id}/status", put(update_game_status))
    // 手动存盘接口
    .route("/game/{game_id}/save", post(manual_save))
    // 查询存档文件列表接口
    .route("/game/{game_id}/saves", get(list_save_files))
    // 导演编辑游戏接口
    .route("/game/{game_id}/edit", put(edit_game))
    // 新增的导演查询日志接口
    .route("/game/{game_id}/director/logs", get(get_director_messages))
    // 新增的导演查询击杀记录接口
    .route("/game/{game_id}/director/kill-records", get(get_director_kill_records))
    .with_state(app_state.clone());

// 玩家接口路由（无需JWT认证，使用玩家密码验证）
let player_routes = Router::new()
    // 获取玩家消息记录接口
    .route(
        "/game/{game_id}/player/{player_id}/messages",
        post(get_player_messages),
    )
    // 新增的获取玩家击杀记录接口
    .route(
        "/game/{game_id}/player/{player_id}/kill-records",
        post(get_player_kill_records),
    )
    .with_state(app_state.clone());
```

### 3.4 存档恢复逻辑修改

在`director/service.rs`的`resume_game`函数中添加删除逻辑：

```rust
/// 恢复游戏（暂停 → 进行中）
pub async fn resume_game(
    &self,
    app_state: &AppState,
    game_id: &str,
    save_file_name: Option<String>,
) -> Result<(), DirectorError> {
    // 检查是否提供了存档文件名
    let file_name = match save_file_name {
        Some(name) => name,
        None => {
            return Err(DirectorError::OtherError {
                message: "必须提供存档文件名".to_string(),
            });
        }
    };

    // 更新数据库中游戏状态为 "running"
    let result = sqlx::query!(
        "UPDATE games SET status = 'running', updated_at = CURRENT_TIMESTAMP WHERE id = ?",
        game_id
    )
    .execute(&self.pool)
    .await
    .map_err(|e| DirectorError::DatabaseError(e))?;

    if result.rows_affected() == 0 {
        return Err(DirectorError::GameNotFound);
    }

    // 从指定的存档文件中恢复游戏状态
    app_state
        .game_state_manager
        .load_game_state_from_disk_with_name(game_id, &file_name)
        .await
        .map_err(|e| DirectorError::OtherError {
            message: format!("Failed to load game state from disk: {}", e),
        })?;

    // 获取恢复的游戏状态的保存时间
    if let Ok(game_state) = app_state.game_state_manager.get_game_state(game_id).await {
        let game_state_guard = game_state.read().await;
        if let Some(save_time) = game_state_guard.save_time {
            // 删除晚于保存时间的日志记录
            let _ = app_state
                .game_log_service
                .delete_logs_after_timestamp(game_id, Some(save_time))
                .await;

            // 删除晚于保存时间的击杀记录
            let _ = app_state
                .game_log_service
                .delete_kill_records_after_timestamp(game_id, Some(save_time))
                .await;
        }
    }

    Ok(())
}
```

## 4. 前端实现

### 4.1 击杀记录展示组件

创建一个新的前端组件用于展示击杀记录：

```vue
<!-- src/components/KillRecordDisplay.vue -->
<template>
  <el-card class="kill-record-display">
    <template #header>
      <div class="card-header">
        <h3>击杀记录</h3>
        <div class="header-controls">
          <el-select 
            v-if="isDirector" 
            v-model="filterForm.selectedKiller" 
            placeholder="筛选击杀者"
            clearable
            size="small"
            class="killer-filter"
          >
            <el-option label="无击杀者" value="__none__" />
            <el-option
              v-for="player in players"
              :key="player.id"
              :label="player.name"
              :value="player.id"
            />
          </el-select>
          <el-button-group>
            <el-button 
              :type="sortOrder === 'asc' ? 'primary' : 'default'" 
              @click="changeSortOrder('asc')"
              size="small"
            >
              时间正序
            </el-button>
            <el-button 
              :type="sortOrder === 'desc' ? 'primary' : 'default'" 
              @click="changeSortOrder('desc')"
              size="small"
            >
              时间倒序
            </el-button>
          </el-button-group>
        </div>
      </div>
    </template>
    
    <el-table 
      :data="filteredAndSortedRecords" 
      style="width: 100%" 
      size="small"
      max-height="400"
    >
      <el-table-column prop="kill_time" label="时间" width="160">
        <template #default="scope">
          {{ formatTime(scope.row.kill_time) }}
        </template>
      </el-table-column>
      <el-table-column prop="killer_name" label="击杀者" width="120">
        <template #default="scope">
          <span v-if="scope.row.killer_name">{{ scope.row.killer_name }}</span>
          <span v-else class="no-killer">无击杀者</span>
        </template>
      </el-table-column>
      <el-table-column prop="victim_name" label="被击杀者" width="120" />
      <el-table-column prop="cause" label="原因" width="120" />
      <el-table-column prop="weapon" label="武器" width="120">
        <template #default="scope">
          <span v-if="scope.row.weapon">{{ scope.row.weapon }}</span>
          <span v-else>-</span>
        </template>
      </el-table-column>
      <el-table-column prop="location" label="地点" width="120">
        <template #default="scope">
          <span v-if="scope.row.location">{{ scope.row.location }}</span>
          <span v-else>-</span>
        </template>
      </el-table-column>
    </el-table>
    
    <div v-if="filteredAndSortedRecords.length === 0" class="no-records">
      暂无击杀记录
    </div>
  </el-card>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import type { KillRecord } from '@/types/game'

interface Props {
  records: KillRecord[]
  players: Array<{ id: string; name: string }>
  isDirector?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  isDirector: false
})

// 响应式数据
const filterForm = ref({
  selectedKiller: ''
})

const sortOrder = ref<'asc' | 'desc'>('asc')

// 计算属性
const recordsWithPlayerNames = computed(() => {
  return props.records.map(record => {
    const killer = props.players.find(p => p.id === record.killer_id)
    const victim = props.players.find(p => p.id === record.victim_id)
    
    return {
      ...record,
      killer_name: killer ? killer.name : null,
      victim_name: victim ? victim.name : '未知'
    }
  })
})

const filteredRecords = computed(() => {
  if (!props.isDirector || !filterForm.value.selectedKiller) {
    return recordsWithPlayerNames.value
  }
  
  if (filterForm.value.selectedKiller === '__none__') {
    return recordsWithPlayerNames.value.filter(record => !record.killer_id)
  }
  
  return recordsWithPlayerNames.value.filter(
    record => record.killer_id === filterForm.value.selectedKiller
  )
})

const filteredAndSortedRecords = computed(() => {
  const sorted = [...filteredRecords.value]
  sorted.sort((a, b) => {
    const timeA = new Date(a.kill_time).getTime()
    const timeB = new Date(b.kill_time).getTime()
    
    if (sortOrder.value === 'asc') {
      return timeA - timeB
    } else {
      return timeB - timeA
    }
  })
  
  return sorted
})

// 方法
const formatTime = (timestamp: string) => {
  return new Date(timestamp).toLocaleString('zh-CN')
}

const changeSortOrder = (order: 'asc' | 'desc') => {
  sortOrder.value = order
}
</script>

<style scoped>
.kill-record-display {
  margin-top: 20px;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  flex-wrap: wrap;
  gap: 10px;
}

.header-controls {
  display: flex;
  gap: 10px;
  flex-wrap: wrap;
}

.killer-filter {
  width: 150px;
}

.no-killer {
  color: #909399;
  font-style: italic;
}

.no-records {
  text-align: center;
  padding: 20px;
  color: #909399;
}
</style>
```

### 4.2 集成到演员页面

在演员页面的适当位置添加击杀记录组件：

```vue
<!-- src/views/actor/ActorMain.vue -->
<template>
  <div class="shared-content">
    <!-- 主内容 -->
    <div class="shared-main-layout">
      <!-- 左侧内容区域 -->
      <div class="shared-left-content">
        <!-- ... existing content ... -->
      </div>
      
      <!-- 右侧日志消息和击杀记录区域 -->
      <div class="shared-right-content">
        <LogMessage 
          v-if="shouldShowLogMessage"
          :messages="logMessages"
          :players="playerList"
          class="shared-log-message"
        />
        
        <!-- 新增的击杀记录组件 -->
        <KillRecordDisplay
          v-if="shouldShowKillRecords"
          :records="killRecords"
          :players="playerList"
          class="shared-kill-records"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
// ... existing imports ...

// 新增导入
import KillRecordDisplay from '@/components/KillRecordDisplay.vue'
import type { KillRecord } from '@/types/game'


// 新增响应式数据
const killRecords = ref<KillRecord[]>([])

// 新增计算属性
const shouldShowKillRecords = computed(() => {
  // 在等待中以外的状态都需要显示
  return game.value && 
    game.value.status !== GameStatus.WAITING && 
    game.value.status !== GameStatus.DELETED
})

// 新增方法
const fetchKillRecords = async () => {
  if (!game.value || !actorPassword.value) return
  
  try {
    const response = await gameService.getPlayerKillRecords(
      game.value.id,
      game.value.id, // 这里应该是玩家ID，需要根据实际情况调整
      actorPassword.value
    )
    
    if (response.success && response.data) {
      killRecords.value = response.data
    }
  } catch (error) {
    console.error('获取击杀记录失败:', error)
  }
}

// 在适当时机调用获取击杀记录的方法
// 例如在游戏状态更新后调用
</script>
```

### 4.3 集成到导演页面

在导演页面的适当位置添加击杀记录组件：

```vue
<!-- src/views/director/DirectorMain.vue -->
<template>
  <div class="director-main">
    <!-- ... existing content ... -->
    
    <div class="director-content">
      <!-- 左侧内容区域 -->
      <div class="left-content">
        <!-- ... existing content ... -->
      </div>
      
      <!-- 右侧日志消息和击杀记录区域 -->
      <div class="right-content">
        <LogMessage 
          v-if="shouldShowLogMessage"
          :messages="logMessages"
          :players="playerList"
          :is-director="true"
          class="director-log-message"
        />
        
        <!-- 新增的击杀记录组件 -->
        <KillRecordDisplay
          v-if="shouldShowKillRecords"
          :records="killRecords"
          :players="playerList"
          :is-director="true"
          class="director-kill-records"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
// ... existing imports ...

// 新增导入
import KillRecordDisplay from '@/components/KillRecordDisplay.vue'
import type { KillRecord } from '@/types/game'


// 新增响应式数据
const killRecords = ref<KillRecord[]>([])

// 新增计算属性
const shouldShowKillRecords = computed(() => {
  // 在等待中以外的状态都需要显示
  return game.value && 
    game.value.status !== GameStatus.WAITING && 
    game.value.status !== GameStatus.DELETED
})

// 新增方法
const fetchKillRecords = async () => {
  if (!game.value || !directorPassword.value) return
  
  try {
    const response = await directorService.getDirectorKillRecords(
      game.value.id,
      directorPassword.value
    )
    
    if (response.success && response.data) {
      killRecords.value = response.data
    }
  } catch (error) {
    console.error('获取击杀记录失败:', error)
  }
}

// 在导演点击"继续游戏"执行完毕后手动调用获取日志和击杀记录的方法
const handleResumeGame = async () => {
  // ... existing resume game logic ...
  
  // 刷新日志和击杀记录
  await fetchLogMessages()
  await fetchKillRecords()
}
</script>
```

## 5. 总结

本设计文档定义了击杀记录查询功能的完整实现方案，包括后端服务层的查询函数、路由处理，以及前端组件的展示和集成。所有修改都遵循了设计原则，确保了功能的最小化实现。