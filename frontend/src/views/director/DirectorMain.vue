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

          <!-- 管理页面内容 -->
          <component 
            :is="currentManagementComponent" 
            :game="gameWithData"
            :director-password="directorPassword"
            @refresh="refreshGame"
            ref="managementComponentRef"
          />
        </div>

        <!-- 右侧日志消息区域 -->
        <div class="shared-right-content">
          <LogMessage 
            v-if="shouldShowLogMessage"
            :messages="logMessages"
            :players="playerList"
            :is-director="true"
            class="shared-log-message"
            @reply-to-player="handleReplyToPlayer"
            @show-kill-records="showKillRecordsDialog"
          />
        </div>
      </div>
    </div>
  </div>
  
  <!-- 击杀记录对话框 -->
  <el-dialog
    v-model="killRecordsDialogVisible"
    title="击杀记录"
    width="80%"
    max-height="80%"
  >
    <KillRecordDisplay
      :records="killRecords"
      :players="playerList"
      :is-director="true"
    />
  </el-dialog>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from 'vue'
import { useRoute } from 'vue-router'
import { ElMessage, ElMessageBox, ElDialog } from 'element-plus'
import { gameService } from '@/services/gameService'
import { useGameStateStore } from '@/stores/gameState'
import type { GameWithRules } from '@/types/game'
import { GameStatus } from '@/types/game'
import type { KillRecord, MessageRecord } from '@/types/game'

// 组件导入 - 使用正确的相对路径
import Header from './components/Header.vue'
import PreGameManagement from './management/PreGameManagement.vue'
import InGameManagement from './management/InGameManagement.vue'
import PostGameManagement from './management/PostGameManagement.vue'
import LogMessage from '@/components/LogMessage.vue'
import KillRecordDisplay from '@/components/KillRecordDisplay.vue'

// 引入公用样式
import '@/styles/director-actor-layout.css'

const route = useRoute()
const gameStateStore = useGameStateStore()

// 添加管理组件引用
const managementComponentRef = ref<any>(null)

// 响应式状态
const game = ref<GameWithRules | null>(null)
const loading = ref(true)
const error = ref<string | null>(null)
const directorPassword = ref<string>('')
const killRecords = ref<KillRecord[]>([])
const killRecordsDialogVisible = ref(false)
const initialMessagesLoaded = ref(false) // 新增状态，标记初始消息是否已加载

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
  // 匹配 /game/{gameId}/director/{password}
  const match = route.fullPath.match(/\/game\/([^/]+)\/director\/([^/]+)$/)
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
      
      // 获取导演消息和击杀记录
      await fetchDirectorMessages()
      await fetchDirectorKillRecords()
      
      // 根据游戏状态处理WebSocket连接
      if (response.data.status === GameStatus.RUNNING && directorPassword.value) {
        // 如果游戏处于进行中状态，建立WebSocket连接（不阻塞页面加载）
        // 只有在之前没有连接时才连接
        if (!webSocketConnected.value) {
          // 异步连接WebSocket，不阻塞页面渲染
          connectWebSocket()
        }
      } else if (response.data.status === GameStatus.PAUSED && webSocketConnected.value) {
        // 如果游戏处于暂停状态且当前已连接，断开WebSocket连接
        gameStateStore.disconnect()
        ElMessage.success('游戏已暂停，WebSocket连接已断开')
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
    await gameStateStore.connect(gameId.value, directorPassword.value, 'director')
  } catch (err) {
    console.error('WebSocket连接失败:', err)
    ElMessage.error('连接游戏服务器失败')
  }
}

// 新增方法：获取导演消息
const fetchDirectorMessages = async () => {
  if (!game.value || !directorPassword.value) return
  
  try {
    const response = await gameService.getDirectorMessages(
      game.value.id,
      directorPassword.value
    )
    
    if (response.success && response.data) {
      // 将API获取的消息与WebSocket消息结合
      // 清空现有的日志消息
      gameStateStore.clearLogMessages()
      
      // 添加API获取的消息到日志消息列表
      response.data.forEach(message => {
        gameStateStore.addLogMessage({
          timestamp: message.timestamp,
          log_message: message.message,
          message_type: message.type
        })
      })
      
      initialMessagesLoaded.value = true
    }
  } catch (error) {
    console.error('获取导演消息失败:', error)
  }
}

// 新增方法：获取导演击杀记录
const fetchDirectorKillRecords = async () => {
  if (!game.value || !directorPassword.value) return
  
  try {
    const response = await gameService.getDirectorKillRecords(
      game.value.id,
      directorPassword.value
    )
    
    if (response.success && response.data) {
      killRecords.value = response.data
    }
  } catch (error) {
    console.error('获取导演击杀记录失败:', error)
  }
}

const refreshGame = async () => {
  await fetchGameDetail()
}

// 新增方法：处理回复玩家事件
const handleReplyToPlayer = (playerId: string) => {
  // 检查管理组件引用是否存在
  if (managementComponentRef.value) {
    // 调用管理组件中的方法来设置目标玩家
    if (typeof managementComponentRef.value.setBroadcastTarget === 'function') {
      managementComponentRef.value.setBroadcastTarget(playerId);
    }
  }
}

// 监听WebSocket错误 - 修复类型问题
watch(webSocketError, (newError: string | null) => {
  if (newError) {
    ElMessage.error(`WebSocket错误: ${newError}`)
    gameStateStore.clearError()
  }
})

// 新增方法：显示击杀记录对话框
const showKillRecordsDialog = async () => {
  // 获取最新的击杀记录
  await fetchDirectorKillRecords()
  // 显示对话框
  killRecordsDialogVisible.value = true
}

// 新增方法：处理恢复游戏后的日志和击杀记录刷新
const handleResumeGame = async () => {
  // 清空并重新获取初始消息列表
  initialMessagesLoaded.value = false
  await fetchDirectorMessages()
  await fetchDirectorKillRecords()
}

// 新增方法：在游戏状态更新时检查是否需要重新加载消息
const handleStatusUpdated = async () => {
  // 状态更新后刷新游戏信息
  await refreshGame()
  
  // 如果是恢复游戏操作，重新加载消息
  if (game.value?.status === GameStatus.RUNNING && !initialMessagesLoaded.value) {
    await fetchDirectorMessages()
  }
}
</script>

<style scoped>
/* 移除了共享样式，现在使用公用CSS文件中的样式 */
</style>
