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

      <!-- 根据游戏状态显示不同的管理页面 -->
      <component 
        :is="currentManagementComponent" 
        :game="game" 
        :director-password="directorPassword"
        @refresh="refreshGame"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { ElMessage } from 'element-plus'
import { gameService } from '@/services/gameService'
import type { GameWithRules } from '@/types/game'
import { GameStatus } from '@/types/game'

// 组件导入
import Header from './components/Header.vue'
import PreGameManagement from './management/PreGameManagement.vue'
import InGameManagement from './management/InGameManagement.vue'
import PostGameManagement from './management/PostGameManagement.vue'

const route = useRoute()
const router = useRouter()

// 响应式状态
const game = ref<GameWithRules | null>(null)
const loading = ref(true)
const error = ref<string | null>(null)
const directorPassword = ref<string>('')

// 计算属性
const gameId = computed(() => route.params.id as string)

// 根据游戏状态映射到对应的管理组件
const currentManagementComponent = computed(() => {
  if (!game.value) return null
  
  const statusComponentMap: Record<string, any> = {
    [GameStatus.WAITING]: PreGameManagement,
    [GameStatus.RUNNING]: InGameManagement,
    [GameStatus.PAUSED]: InGameManagement,
    [GameStatus.ENDED]: PostGameManagement,
    [GameStatus.HIDDEN]: PostGameManagement,
    [GameStatus.DELETED]: PostGameManagement
  }
  
  return statusComponentMap[game.value.status] || PostGameManagement
})

// 生命周期
onMounted(() => {
  // 检查是否从URI中获取密码
  checkURIPassword()
  // 获取游戏详情
  fetchGameDetail()
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

const refreshGame = async () => {
  await fetchGameDetail()
}

const handleStatusUpdated = () => {
  // 状态更新后刷新游戏信息
  refreshGame()
}
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

.error-alert {
  max-width: 1200px;
  margin: 0 auto;
}

.director-content {
  max-width: 1200px;
  margin: 0 auto;
}

@media (max-width: 768px) {
  .director-main {
    padding: 16px;
  }
}
</style>