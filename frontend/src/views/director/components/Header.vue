<template>
  <div class="director-header">
    <el-card class="header-card">
      <template #header>
        <div class="card-header">
          <div class="header-info">
            <h2>{{ game.name }}</h2>
            <div class="game-status">
              <el-tag :type="statusTagType" size="large">
                {{ statusDisplayText }}
              </el-tag>
              <span class="game-id">游戏ID: {{ game.id }}</span>
            </div>
          </div>
          <div class="header-actions">
            <el-button 
              v-if="showStartButton"
              type="primary"
              @click="confirmStartGame"
              :loading="actionLoading"
            >
              开始游戏
            </el-button>
            <el-button 
              v-if="showPauseButton"
              type="warning"
              @click="pauseGame"
              :loading="actionLoading"
            >
              暂停游戏
            </el-button>
            <el-button 
              v-if="showResumeButton"
              type="success"
              @click="resumeGame"
              :loading="actionLoading"
            >
              继续游戏
            </el-button>
            <el-button 
              v-if="showEndButton"
              type="danger"
              @click="confirmEndGame"
              :loading="actionLoading"
            >
              结束游戏
            </el-button>
            <el-button @click="goBack">
              返回游戏详情
            </el-button>
          </div>
        </div>
      </template>
      
      <div class="game-info">
        <p v-if="game.description">{{ game.description }}</p>
        <p v-else class="no-description">暂无游戏描述</p>
        <div class="game-stats">
          <span>演员数量: {{ game.player_count }} / {{ game.max_players }}</span>
          <span>创建时间: {{ formatDate(game.created_at) }}</span>
        </div>
      </div>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRouter } from 'vue-router'
import { ElMessage, ElMessageBox } from 'element-plus'
import type { GameWithRules } from '@/types/game'
import { GameStatus } from '@/types/game'
import apiClient from '@/services/client'
import { API_ENDPOINTS } from '@/services/config'

// Props
const props = defineProps<{
  game: GameWithRules
  directorPassword: string
}>()

// Emits
const emit = defineEmits<{
  (e: 'status-updated'): void
}>()

// Router
const router = useRouter()

// 响应式状态
const actionLoading = ref(false)

// 计算属性
const statusDisplayText = computed(() => {
  const statusMap: Record<string, string> = {
    [GameStatus.WAITING]: '等待中',
    [GameStatus.RUNNING]: '进行中',
    [GameStatus.PAUSED]: '已暂停',
    [GameStatus.ENDED]: '已结束',
    [GameStatus.HIDDEN]: '已隐藏',
    [GameStatus.DELETED]: '已删除'
  }
  return statusMap[props.game.status] || props.game.status
})

const statusTagType = computed(() => {
  const typeMap: Record<string, 'info' | 'success' | 'warning' | 'danger'> = {
    [GameStatus.WAITING]: 'info',
    [GameStatus.RUNNING]: 'success',
    [GameStatus.PAUSED]: 'warning',
    [GameStatus.ENDED]: 'danger',
    [GameStatus.HIDDEN]: 'danger',
    [GameStatus.DELETED]: 'danger'
  }
  return typeMap[props.game.status] || 'info'
})

const showStartButton = computed(() => props.game.status === GameStatus.WAITING)
const showPauseButton = computed(() => props.game.status === GameStatus.RUNNING)
const showResumeButton = computed(() => props.game.status === GameStatus.PAUSED)
const showEndButton = computed(() => 
  props.game.status === GameStatus.RUNNING || 
  props.game.status === GameStatus.PAUSED
)

// 方法实现
const formatDate = (dateString: string) => {
  return new Date(dateString).toLocaleString('zh-CN')
}

const updateGameStatus = async (targetStatus: GameStatus) => {
  if (!props.directorPassword) {
    ElMessage.error('缺少导演密码')
    return
  }
  
  actionLoading.value = true
  
  try {
    const response = await apiClient.put(
      `/game/${props.game.id}/status`,
      { password: props.directorPassword, status: targetStatus }
    )
    
    if (response.data.success) {
      ElMessage.success('游戏状态更新成功')
      emit('status-updated')
    } else {
      throw new Error(response.data.message || '状态更新失败')
    }
  } catch (error: any) {
    console.error('更新游戏状态失败:', error)
    ElMessage.error(
      error.response?.status === 401 
        ? '导演密码错误' 
        : (error.message || '状态更新失败')
    )
  } finally {
    actionLoading.value = false
  }
}

const confirmStartGame = () => {
  ElMessageBox.confirm(
    '确定要开始游戏吗？开始后无法回到等待状态。',
    '确认开始游戏',
    {
      confirmButtonText: '确定',
      cancelButtonText: '取消',
      type: 'warning'
    }
  ).then(() => {
    updateGameStatus(GameStatus.RUNNING)
  }).catch(() => {
    // 用户取消操作
  })
}

const pauseGame = () => {
  updateGameStatus(GameStatus.PAUSED)
}

const resumeGame = () => {
  updateGameStatus(GameStatus.RUNNING)
}

const confirmEndGame = () => {
  ElMessageBox.confirm(
    '确定要结束游戏吗？结束后无法重新开始。',
    '确认结束游戏',
    {
      confirmButtonText: '确定',
      cancelButtonText: '取消',
      type: 'warning'
    }
  ).then(() => {
    updateGameStatus(GameStatus.ENDED)
  }).catch(() => {
    // 用户取消操作
  })
}

const goBack = () => {
  router.push(`/game/${props.game.id}`)
}
</script>

<style scoped>
.director-header {
  margin-bottom: 24px;
}

.header-card {
  border-radius: 8px;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  flex-wrap: wrap;
  gap: 16px;
}

.header-info h2 {
  margin: 0 0 8px 0;
  color: #303133;
  font-size: 24px;
}

.game-status {
  display: flex;
  align-items: center;
  gap: 16px;
  flex-wrap: wrap;
}

.game-id {
  color: #909399;
  font-size: 14px;
}

.header-actions {
  display: flex;
  gap: 12px;
  flex-wrap: wrap;
}

.game-info p {
  margin: 0 0 12px 0;
  color: #606266;
  line-height: 1.6;
}

.no-description {
  font-style: italic;
  color: #C0C4CC;
}

.game-stats {
  display: flex;
  gap: 24px;
  color: #909399;
  font-size: 14px;
  flex-wrap: wrap;
}

@media (max-width: 768px) {
  .card-header {
    flex-direction: column;
    align-items: stretch;
  }
  
  .header-actions {
    justify-content: flex-end;
  }
  
  .game-status {
    flex-direction: column;
    align-items: flex-start;
    gap: 8px;
  }
  
  .header-info h2 {
    font-size: 20px;
  }
}
</style>