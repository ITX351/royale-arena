<template>
  <div class="director-main">
    <!-- 加载状态 -->
    <div v-if="loading" class="loading-container">
      <el-skeleton animated>
        <template #template>
          <el-skeleton-item variant="h3" style="width: 30%" />
          <div style="margin-top: 20px">
            <el-skeleton-item variant="p" style="width: 50%" />
          </div>
        </template>
      </el-skeleton>
    </div>

    <!-- 错误状态 -->
    <el-alert
      v-else-if="error"
      :title="error"
      type="error"
      show-icon
      :closable="false"
      class="error-alert"
    />

    <!-- 主内容 -->
    <div v-else-if="game" class="director-content">
      <!-- 题头组件 -->
      <Header 
        :game="game" 
        :director-password="directorPassword"
        @status-updated="handleStatusUpdated"
      />

      <!-- WebSocket连接状态提示 -->
      <el-alert
        v-if="webSocketConnecting"
        title="正在连接到游戏服务器..."
        type="info"
        show-icon
        :closable="false"
        class="connection-alert"
      />

      <el-alert
        v-else-if="!webSocketConnected && game.status === GameStatus.RUNNING"
        title="WebSocket连接已断开，正在尝试重连..."
        type="warning"
        show-icon
        :closable="false"
        class="connection-alert"
      />

      <!-- 根据游戏状态显示不同的管理页面 -->
      <component 
        :is="currentManagementComponent" 
        :game="gameWithData"
        :director-password="directorPassword"
        @refresh="refreshGame"
        @request-pause="handleRequestPause"
        @request-end="handleRequestEnd"
      />

      <!-- 实时日志消息组件 -->
      <LogMessage 
        v-if="game.status === GameStatus.RUNNING || game.status === GameStatus.PAUSED"
        :messages="logMessages"
        :players="playerList"
        class="log-message-component"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { ElMessage, ElMessageBox } from 'element-plus'
import { gameService } from '@/services/gameService'
import { useGameStateStore } from '@/stores/gameState'
import type { GameWithRules } from '@/types/game'
import { GameStatus } from '@/types/game'

// 组件导入 - 使用正确的相对路径
import Header from './components/Header.vue'
import PreGameManagement from './management/PreGameManagement.vue'
import InGameManagement from './management/InGameManagement.vue'
import PostGameManagement from './management/PostGameManagement.vue'
import LogMessage from './components/LogMessage.vue'

const route = useRoute()
const router = useRouter()
const gameStateStore = useGameStateStore()

// 响应式状态
const game = ref<GameWithRules | null>(null)
const loading = ref(true)
const error = ref<string | null>(null)
const directorPassword = ref<string>('')

// 计算属性
const gameId = computed(() => route.params.id as string)

const webSocketConnected = computed(() => gameStateStore.connected)
const webSocketConnecting = computed(() => gameStateStore.connecting)
const webSocketError = computed(() => gameStateStore.error)
const gameStateData = computed(() => gameStateStore.gameState)
const logMessages = computed(() => gameStateStore.logMessages)
const playerList = computed(() => {
  if (!game.value) return [] as { id: string; name: string }[]
  // 修复类型问题：确保返回正确的类型
  return [] as { id: string; name: string }[]
})

// 合并游戏数据和实时状态数据
const gameWithData = computed(() => {
  if (!game.value) return null
  
  // 如果有实时状态数据，合并到游戏数据中
  if (gameStateData.value) {
    return {
      ...game.value,
      ...gameStateData.value.game_data,
      global_state: gameStateData.value.global_state
    }
  }
  
  return game.value
})

// 根据游戏状态映射到对应的管理组件
const currentManagementComponent = computed(() => {
  if (!gameWithData.value) return null
  
  const statusComponentMap: Record<string, any> = {
    [GameStatus.WAITING]: PreGameManagement,
    [GameStatus.RUNNING]: InGameManagement,
    [GameStatus.PAUSED]: InGameManagement,
    [GameStatus.ENDED]: PostGameManagement,
    [GameStatus.HIDDEN]: PostGameManagement,
    [GameStatus.DELETED]: PostGameManagement
  }
  
  return statusComponentMap[gameWithData.value.status] || PostGameManagement
})

// 生命周期
onMounted(() => {
  // 检查是否从URI中获取密码
  checkURIPassword()
  // 获取游戏详情
  fetchGameDetail()
})

onUnmounted(() => {
  // 组件卸载时断开WebSocket连接
  gameStateStore.disconnect()
})

// 方法实现
const checkURIPassword = () => {
  // 匹配 /game/{gameId}/{password}
  const match = route.fullPath.match(/\/game\/([^/]+)\/([^/]+)$/)
  if (match) {
    directorPassword.value = decodeURIComponent(match[2])
  }
}

const fetchGameDetail = async () => {
  loading.value = true
  error.value = null
  
  try {
    const response = await gameService.getGameDetail(gameId.value)
    if (response.success && response.data) {
      game.value = response.data
      
      // 如果游戏处于进行中状态，建立WebSocket连接（不阻塞页面加载）
      if (response.data.status === GameStatus.RUNNING && directorPassword.value) {
        // 异步连接WebSocket，不阻塞页面渲染
        connectWebSocket()
      }
    } else {
      throw new Error(response.message || '获取游戏详情失败')
    }
  } catch (err: any) {
    console.error('获取游戏详情失败:', err)
    error.value = err.response?.status === 404 
      ? '游戏不存在' 
      : (err.message || '获取游戏详情失败')
  } finally {
    loading.value = false
  }
}

const connectWebSocket = async () => {
  if (!game.value || !directorPassword.value) return
  
  try {
    await gameStateStore.connect(gameId.value, directorPassword.value)
  } catch (err) {
    console.error('WebSocket连接失败:', err)
    ElMessage.error('连接游戏服务器失败')
  }
}

const refreshGame = async () => {
  await fetchGameDetail()
}

const handleStatusUpdated = () => {
  // 状态更新后刷新游戏信息
  refreshGame()
}

const handleRequestPause = async () => {
  ElMessageBox.confirm(
    '确定要暂停游戏吗？',
    '暂停游戏',
    {
      confirmButtonText: '确定',
      cancelButtonText: '取消',
      type: 'warning',
    }
  ).then(() => {
    // 发送暂停游戏指令
    gameStateStore.sendDirectorAction('pause')
    ElMessage.success('暂停游戏指令已发送')
  }).catch(() => {
    // 用户取消操作
  })
}

const handleRequestEnd = async () => {
  ElMessageBox.confirm(
    '确定要结束游戏吗？此操作不可撤销。',
    '结束游戏',
    {
      confirmButtonText: '确定',
      cancelButtonText: '取消',
      type: 'error',
    }
  ).then(() => {
    // 发送结束游戏指令
    gameStateStore.sendDirectorAction('end')
    ElMessage.success('结束游戏指令已发送')
  }).catch(() => {
    // 用户取消操作
  })
}

// 监听WebSocket错误 - 修复类型问题
watch(webSocketError, (newError: string | null) => {
  if (newError) {
    ElMessage.error(`WebSocket错误: ${newError}`)
    gameStateStore.clearError()
  }
})
</script>

<style scoped>
.director-main {
  min-height: 100vh;
  padding: 24px;
  background-color: #f5f7fa;
}

.loading-container {
  max-width: 1200px;
  margin: 0 auto;
  padding: 40px;
  background: white;
  border-radius: 8px;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
}

.error-alert,
.connection-alert {
  max-width: 1200px;
  margin: 0 auto 20px;
}

.director-content {
  max-width: 1200px;
  margin: 0 auto;
}

.log-message-component {
  margin-top: 20px;
}

@media (max-width: 768px) {
  .director-main {
    padding: 16px;
  }
}
</style>