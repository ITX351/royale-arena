<template>
  <el-card class="game-card" :class="getGameStatusClass(game.status)">
    <template #header>
      <div class="card-header">
        <div class="title-section">
          <h3 class="game-title">{{ game.name }}</h3>
          <p class="game-description" v-if="game.description">{{ game.description }}</p>
        </div>
        <el-tag 
          :type="getGameStatusConfig(game.status).type"
          :color="getGameStatusConfig(game.status).color" 
          size="small"
          effect="dark"
        >
          {{ getGameStatusConfig(game.status).text }}
        </el-tag>
      </div>
    </template>
    
    <div class="game-info">
      <!-- 玩家数量信息 -->
      <div class="info-item">
        <el-icon><User /></el-icon>
        <span class="info-label">玩家数量：</span>
        <span class="info-value">{{ game.player_count || 0 }}/{{ game.max_players }}</span>
      </div>
      
      <!-- 创建时间 -->
      <div class="info-item">
        <el-icon><Clock /></el-icon>
        <span class="info-label">创建时间：</span>
        <span class="info-value">{{ formatDateTime(game.created_at) }}</span>
      </div>
    </div>
    
    <!-- 游戏操作区域 -->
    <div class="game-actions">
      <!-- 快捷登录区域 -->
      <div class="quick-login" v-if="canQuickLogin">
        <form @submit.prevent="handleQuickLogin" class="login-form">
          <div class="login-input-group">
            <el-input 
              v-model="loginPassword" 
              type="password"
              placeholder="输入密码快速加入"
              size="small"
              show-password
              @keyup.enter="handleQuickLogin"
            >
              <template #append>
                <el-button 
                  type="primary" 
                  :loading="loginLoading"
                  native-type="submit"
                  size="small"
                >
                  登录
                </el-button>
              </template>
            </el-input>
          </div>
          <div class="login-hint">
            <span class="hint-text">输入密码后自动识别身份进入游戏</span>
          </div>
        </form>
      </div>
      
      <!-- 详情按钮 -->
      <el-button 
        type="info" 
        size="default"
        @click="$emit('view-detail', game)"
        class="detail-button"
        plain
      >
        <el-icon><View /></el-icon>
        查看详情
      </el-button>
    </div>
  </el-card>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRouter } from 'vue-router'
import { ElMessage } from 'element-plus'
import { User, Clock, Document, View } from '@element-plus/icons-vue'
import type { GameListItem } from '@/types/game'
import { getGameStatusConfig, formatDateTime } from '@/utils/gameFilter'
import { directorService } from '@/services/directorService'
import { authenticateGame, handleAuthResult } from '@/services/authService'

interface Props {
  game: GameListItem
}

const props = defineProps<Props>()
const router = useRouter()

// 响应式数据
const loginPassword = ref('')
const loginLoading = ref(false)

// 计算属性
const canQuickLogin = computed(() => {
  return props.game.status === 'waiting' || 
         props.game.status === 'running' ||
         props.game.status === 'paused'
})

// 快捷登录处理
const handleQuickLogin = async () => {
  loginLoading.value = true
  
  try {
    // 使用统一的认证服务
    const authResult = await authenticateGame(props.game.id, loginPassword.value)
    handleAuthResult(authResult, props.game.id, loginPassword.value, router)
  } finally {
    loginLoading.value = false
    if (!loginPassword.value) {
      loginPassword.value = ''
    }
  }
}

// 获取游戏状态样式类
const getGameStatusClass = (status: string) => {
  return `game-card--${status}`
}
</script>

<style scoped>
.game-info {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-bottom: 16px;
}

.info-item {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
}

.info-label {
  color: #606266;
  min-width: 60px;
}

.info-value {
  color: #303133;
  font-weight: 500;
}

.game-actions {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.quick-login {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.login-form {
  width: 100%;
}

.login-input-group {
  width: 100%;
}

.login-hint {
  text-align: center;
}

.hint-text {
  font-size: 11px;
  color: #909399;
  line-height: 1.2;
}
</style>