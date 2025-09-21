<template>
  <div class="director-header">
    <el-card class="header-card" :class="{ 'hide-body': !showDetails }">
      <template #header>
        <div class="card-header">
          <div class="header-title">
            <h2>{{ game.name }}</h2>
            <el-button 
              type="primary" 
              link 
              @click="showDetails = !showDetails"
              :icon="showDetails ? ArrowUp : ArrowDown"
              class="toggle-button"
            />
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
            <!-- <el-button @click="goBack">
              返回游戏详情
            </el-button> -->
            <el-button @click="goHome">
              返回首页
            </el-button>
          </div>
        </div>
        <div class="game-status-line">
          <el-tag :type="statusTagType" size="large">
            {{ statusDisplayText }}
          </el-tag>
          <span>演员人数: {{ game.player_count }}</span>
        </div>
      </template>
      
      <div v-show="showDetails" class="game-details">
        <p v-if="game.description">{{ game.description }}</p>
        <p v-else class="no-description">暂无游戏描述</p>
        <div class="game-stats">
          <span>创建时间: {{ formatDate(game.created_at) }}</span>
          <span class="game-id">游戏ID: {{ game.id }}</span>
        </div>
      </div>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRouter } from 'vue-router'
import { ElMessage, ElMessageBox } from 'element-plus'
import { ArrowDown, ArrowUp } from '@element-plus/icons-vue'
import type { GameWithRules } from '@/types/game'
import { GameStatus } from '@/types/game'
import apiClient from '@/services/client'
import { API_ENDPOINTS } from '@/services/config'
import { 
  formatGameStatus, 
  getStatusTagType,
  shouldShowStartButton,
  shouldShowPauseButton,
  shouldShowResumeButton,
  shouldShowEndButton
} from '@/utils/gameUtils'

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
const showDetails = ref(false)

// 计算属性
const statusDisplayText = computed(() => {
  return formatGameStatus(props.game.status)
})

const statusTagType = computed(() => {
  return getStatusTagType(props.game.status)
})

const showStartButton = computed(() => shouldShowStartButton(props.game.status))
const showPauseButton = computed(() => shouldShowPauseButton(props.game.status))
const showResumeButton = computed(() => shouldShowResumeButton(props.game.status))
const showEndButton = computed(() => shouldShowEndButton(props.game.status))

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
  ElMessageBox.confirm(
    '确定要暂停游戏吗？',
    '暂停游戏',
    {
      confirmButtonText: '确定',
      cancelButtonText: '取消',
      type: 'warning',
    }
  ).then(() => {
    // 更新游戏状态为暂停
    updateGameStatus(GameStatus.PAUSED)
    // 发送暂停游戏指令
    // 注意：这里我们不直接断开WebSocket连接，而是在DirectorMain中监听状态变化
    ElMessage.success('游戏已暂停')
  }).catch(() => {
    // 用户取消操作
  })
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
    ElMessage.success('游戏已结束')
  }).catch(() => {
    // 用户取消操作
  })
}

const goBack = () => {
  router.push(`/game/${props.game.id}`)
}

const goHome = () => {
  router.push('/')
}
</script>

<style scoped>
@import '@/styles/shared-header.css';

.director-header {
  margin-bottom: 24px;
}

.header-actions {
  display: flex;
  gap: 12px;
  flex-wrap: wrap;
  flex-shrink: 0;
}
</style>