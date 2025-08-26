<template>
  <div class="game-detail-page">
    <div class="container">
      <!-- 返回按钮 -->
      <div class="back-section">
        <el-button @click="goBack" :icon="ArrowLeft" plain>
          返回首页
        </el-button>
      </div>

      <!-- 加载状态 -->
      <div v-if="loading" class="loading-section">
        <el-skeleton :rows="8" animated />
      </div>

      <!-- 错误状态 -->
      <el-alert
        v-else-if="error"
        :title="error"
        type="error"
        :closable="false"
        class="error-alert"
      />

      <!-- 游戏详情 -->
      <div v-else-if="gameDetail" class="game-detail">
        <!-- 游戏基本信息 -->
        <el-card class="info-card">
          <template #header>
            <div class="card-header">
              <h2 class="game-title">{{ gameDetail.name }}</h2>
              <el-tag 
                :type="getGameStatusConfig(gameDetail.status).type"
                size="large"
                effect="dark"
              >
                {{ getGameStatusConfig(gameDetail.status).text }}
              </el-tag>
            </div>
          </template>

          <div class="game-info-grid">
            <div class="info-item">
              <div class="info-label">
                <el-icon><InfoFilled /></el-icon>
                游戏描述
              </div>
              <div class="info-value">
                {{ gameDetail.description || '暂无描述' }}
              </div>
            </div>

            <div class="info-item">
              <div class="info-label">
                <el-icon><User /></el-icon>
                玩家数量
              </div>
              <div class="info-value">
                {{ gameDetail.player_count }}/{{ gameDetail.max_players }}
              </div>
            </div>

            <div class="info-item">
              <div class="info-label">
                <el-icon><Clock /></el-icon>
                创建时间
              </div>
              <div class="info-value">
                {{ formatDateTime(gameDetail.created_at) }}
              </div>
            </div>

            <div class="info-item" v-if="gameDetail.rule_template">
              <div class="info-label">
                <el-icon><Document /></el-icon>
                规则模版
              </div>
              <div class="info-value">
                {{ gameDetail.rule_template.template_name }}
              </div>
            </div>
          </div>
        </el-card>

        <!-- 登录区域 -->
        <el-card class="login-card" v-if="canLogin">
          <template #header>
            <h3>
              <el-icon><Key /></el-icon>
              加入游戏
            </h3>
          </template>

          <div class="login-form">
            <el-input
              v-model="loginPassword"
              type="password"
              placeholder="请输入游戏密码"
              size="large"
              show-password
              @keyup.enter="handleLogin"
            >
              <template #prefix>
                <el-icon><Lock /></el-icon>
              </template>
            </el-input>
            
            <el-button
              type="primary"
              size="large"
              :loading="loginLoading"
              @click="handleLogin"
              class="login-button"
            >
              <el-icon><Right /></el-icon>
              登录进入游戏
            </el-button>
            
            <div class="login-hint">
              <el-icon><QuestionFilled /></el-icon>
              输入密码后系统将自动识别您的身份（玩家或导演）
            </div>
          </div>
        </el-card>

        <!-- 游戏状态提示 -->
        <el-card v-else class="status-card">
          <div class="status-message">
            <el-icon class="status-icon"><WarningFilled /></el-icon>
            <span>{{ getStatusMessage() }}</span>
          </div>
        </el-card>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { ElMessage } from 'element-plus'
import { 
  ArrowLeft, 
  InfoFilled, 
  User, 
  Clock, 
  Document, 
  Key, 
  Lock, 
  Right, 
  QuestionFilled,
  WarningFilled
} from '@element-plus/icons-vue'
import type { GameWithRules } from '@/types/game'
import { getGameStatusConfig, formatDateTime } from '@/utils/gameFilter'
import { gameService } from '@/services/gameService'

const route = useRoute()
const router = useRouter()

// 响应式数据
const gameDetail = ref<GameWithRules | null>(null)
const loading = ref(false)
const error = ref<string | null>(null)
const loginPassword = ref('')
const loginLoading = ref(false)

// 计算属性
const canLogin = computed(() => {
  if (!gameDetail.value) return false
  
  return ['waiting', 'running', 'paused'].includes(gameDetail.value.status)
})

// 方法
const loadGameDetail = async () => {
  const gameId = route.params.id as string
  if (!gameId) {
    error.value = '无效的游戏ID'
    return
  }

  loading.value = true
  error.value = null

  try {
    const response = await gameService.getGameDetail(gameId)
    if (response.success && response.data) {
      gameDetail.value = response.data
    } else {
      throw new Error(response.error?.message || '获取游戏详情失败')
    }
  } catch (err) {
    console.error('加载游戏详情失败:', err)
    error.value = err instanceof Error ? err.message : '加载游戏详情失败'
  } finally {
    loading.value = false
  }
}

const handleLogin = async () => {
  if (!loginPassword.value.trim()) {
    ElMessage.error('请输入密码')
    return
  }

  if (!gameDetail.value) {
    ElMessage.error('游戏信息加载失败')
    return
  }

  loginLoading.value = true

  try {
    // 先尝试作为玩家登录
    try {
      await gameService.joinAsPlayer(gameDetail.value.id, loginPassword.value)
      ElMessage.success('成功以玩家身份加入游戏')
      router.push(`/game/${gameDetail.value.id}/player`)
      return
    } catch (playerError) {
      // 玩家登录失败，尝试导演登录
      try {
        await gameService.joinAsDirector(gameDetail.value.id, loginPassword.value)
        ElMessage.success('成功以导演身份进入控制台')
        router.push(`/game/${gameDetail.value.id}/director`)
        return
      } catch (directorError) {
        // 两种身份都登录失败
        ElMessage.error('密码错误或无权限')
      }
    }
  } catch (error) {
    console.error('登录失败:', error)
    ElMessage.error('登录失败，请稍后重试')
  } finally {
    loginLoading.value = false
    loginPassword.value = ''
  }
}

const goBack = () => {
  router.push('/')
}

const getStatusMessage = () => {
  if (!gameDetail.value) return ''
  
  switch (gameDetail.value.status) {
    case 'ended':
      return '游戏已结束，无法加入'
    case 'hidden':
      return '游戏已隐藏'
    case 'deleted':
      return '游戏已删除'
    default:
      return '当前游戏状态不允许加入'
  }
}

// 生命周期
onMounted(() => {
  loadGameDetail()
})
</script>

<style scoped>
.game-detail-page {
  min-height: 100vh;
  background-color: #f5f7fa;
  padding: 24px 0;
}

.container {
  max-width: 800px;
  margin: 0 auto;
  padding: 0 16px;
}

.back-section {
  margin-bottom: 24px;
}

.loading-section,
.error-alert {
  margin-bottom: 24px;
}

.game-detail {
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.info-card {
  border-radius: 8px;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 16px;
}

.game-title {
  margin: 0;
  font-size: 24px;
  font-weight: bold;
  color: #303133;
}

.game-info-grid {
  display: grid;
  gap: 20px;
}

.info-item {
  padding: 16px;
  background: #f8f9fa;
  border-radius: 6px;
}

.info-label {
  display: flex;
  align-items: center;
  gap: 8px;
  font-weight: 600;
  color: #606266;
  margin-bottom: 8px;
}

.info-value {
  color: #303133;
  font-size: 16px;
}

.login-card {
  border-radius: 8px;
}

.login-form {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.login-button {
  width: 100%;
}

.login-hint {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 14px;
  color: #909399;
  text-align: center;
  justify-content: center;
}

.status-card {
  border-radius: 8px;
}

.status-message {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  font-size: 16px;
  color: #909399;
  padding: 20px;
}

.status-icon {
  font-size: 20px;
}

/* 响应式设计 */
@media (max-width: 767px) {
  .container {
    padding: 0 12px;
  }
  
  .card-header {
    flex-direction: column;
    align-items: flex-start;
    gap: 12px;
  }
  
  .game-title {
    font-size: 20px;
  }
}

@media (min-width: 768px) {
  .game-info-grid {
    grid-template-columns: repeat(2, 1fr);
  }
}
</style>