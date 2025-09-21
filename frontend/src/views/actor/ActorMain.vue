<template>
  <div class="shared-main">
    <!-- 加载状态 -->
    <div v-if="loading" class="shared-loading-container">
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
      class="shared-alert"
    />

    <!-- 主内容 -->
    <div v-else-if="game" class="shared-content">
      <!-- 左右分栏布局 -->
      <div class="shared-main-layout">
        <!-- 左侧内容区域 -->
        <div class="shared-left-content">
          <!-- 题头组件 -->
          <Header 
            :game="game" 
            :actor-password="actorPassword"
          />

          <!-- WebSocket连接状态提示 -->
          <el-alert
            v-if="webSocketConnecting"
            title="正在连接到游戏服务器..."
            type="info"
            show-icon
            :closable="false"
            class="shared-alert"
          />

          <el-alert
            v-else-if="!webSocketConnected && game.status === GameStatus.RUNNING"
            title="WebSocket连接已断开，正在尝试重连..."
            type="warning"
            show-icon
            :closable="false"
            class="shared-alert"
          />

          <!-- 状态页面内容 -->
          <component 
            :is="currentStateComponent" 
            :game="gameWithData"
            :actor-password="actorPassword"
          />
        </div>

        <!-- 右侧日志消息区域 -->
        <div class="shared-right-content">
          <LogMessage 
            v-if="shouldShowLogMessage"
            :messages="logMessages"
            :players="playerList"
            class="shared-log-message"
          />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { ElMessage } from 'element-plus'
import { gameService } from '@/services/gameService'
import { useGameStateStore } from '@/stores/gameState'
import type { GameWithRules } from '@/types/game'
import { GameStatus } from '@/types/game'

// 组件导入
import Header from '@/views/actor/components/Header.vue'
import LogMessage from '@/components/LogMessage.vue'
import PreGameState from './states/PreGameState.vue'
import InGameState from './states/InGameState.vue'
import OtherState from './states/OtherState.vue'

// 引入公用样式
import '@/styles/director-actor-layout.css'

const route = useRoute()
const router = useRouter()
const gameStateStore = useGameStateStore()

// 响应式状态
const game = ref<GameWithRules | null>(null)
const loading = ref(true)
const error = ref<string | null>(null)
const actorPassword = ref<string>('')

// 计算属性
const gameId = computed(() => route.params.id as string)

const webSocketConnected = computed(() => gameStateStore.connected)
const webSocketConnecting = computed(() => gameStateStore.connecting)
const webSocketError = computed(() => gameStateStore.error)
const gameStateData = computed(() => gameStateStore.gameState)
const logMessages = computed(() => gameStateStore.logMessages)
const playerList = computed(() => {
  // 从gameStateStore获取玩家列表
  return gameStateStore.playerList
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

// 根据游戏状态映射到对应的状态组件
const currentStateComponent = computed(() => {
  if (!gameWithData.value) return null
  
  // 状态组件映射关系
  const statusComponentMap: Record<string, any> = {
    [GameStatus.WAITING]: PreGameState,
    [GameStatus.RUNNING]: InGameState,
    [GameStatus.PAUSED]: InGameState,
    [GameStatus.ENDED]: OtherState,
    [GameStatus.HIDDEN]: OtherState,
    [GameStatus.DELETED]: OtherState
  }
  
  return statusComponentMap[gameWithData.value.status] || OtherState
})

// 判断是否应该显示日志消息组件
const shouldShowLogMessage = computed(() => {
  if (!game.value) return false
  return game.value.status !== GameStatus.WAITING;
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
  // 匹配 /game/{gameId}/actor/{password} 或 /game/{gameId}/actor
  const match = route.fullPath.match(/\/game\/([^/]+)\/actor\/?(.*)$/)
  if (match && match[2]) {
    actorPassword.value = decodeURIComponent(match[2])
  }
}

const fetchGameDetail = async () => {
  loading.value = true
  error.value = null
  
  try {
    const response = await gameService.getGameDetail(gameId.value)
    if (response.success && response.data) {
      game.value = response.data
      
      // 根据游戏状态处理WebSocket连接
      if ((response.data.status === GameStatus.RUNNING || response.data.status === GameStatus.PAUSED) && actorPassword.value) {
        // 如果游戏处于进行中或暂停状态，建立WebSocket连接（不阻塞页面加载）
        // 只有在之前没有连接时才连接
        if (!webSocketConnected.value) {
          // 异步连接WebSocket，不阻塞页面渲染
          connectWebSocket()
        }
      } else if (response.data.status !== GameStatus.RUNNING && response.data.status !== GameStatus.PAUSED && webSocketConnected.value) {
        // 如果游戏不处于进行中或暂停状态且当前已连接，断开WebSocket连接
        gameStateStore.disconnect()
        ElMessage.success('游戏状态已变更，WebSocket连接已断开')
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
  if (!game.value || !actorPassword.value) return
  
  try {
    // 连接时指定角色为玩家
    await gameStateStore.connect(gameId.value, actorPassword.value, 'actor')
  } catch (err) {
    console.error('WebSocket连接失败:', err)
    ElMessage.error('连接游戏服务器失败')
  }
}

// 监听WebSocket错误
watch(webSocketError, (newError: string | null) => {
  if (newError) {
    ElMessage.error(`WebSocket错误: ${newError}`)
    gameStateStore.clearError()
  }
})
</script>

<style scoped>
/* 移除了共享样式，现在使用公用CSS文件中的样式 */
</style>