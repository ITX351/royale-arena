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
      
      <!-- 规则模版信息 -->
      <div class="info-item" v-if="game.rule_template">
        <el-icon><Document /></el-icon>
        <span class="info-label">规则模版：</span>
        <span class="info-value">{{ game.rule_template.template_name }}</span>
      </div>
    </div>
    
    <!-- 游戏操作区域 -->
    <div class="game-actions">
      <!-- 快捷登录区域 -->
      <div class="quick-login" v-if="canQuickLogin">
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
                @click="handleQuickLogin"
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
  if (!loginPassword.value.trim()) {
    ElMessage.error('请输入密码')
    return
  }
  
  loginLoading.value = true
  
  try {
    // 直接尝试导演身份验证
    try {
      await directorService.authenticateAndGetPlayers(props.game.id, loginPassword.value)
      ElMessage.success('成功以导演身份进入控制台')
      router.push(`/game/${props.game.id}/${encodeURIComponent(loginPassword.value)}`)
      return
    } catch (directorError) {
      // 导演登录失败，显示错误信息
      console.error('导演登录失败:', directorError)
      ElMessage.error('密码错误或无权限')
    }
  } catch (error) {
    console.error('登录失败:', error)
    ElMessage.error('登录失败，请稍后重试')
  } finally {
    loginLoading.value = false
    loginPassword.value = ''
  }
}

// 获取游戏状态样式类
const getGameStatusClass = (status: string) => {
  return `game-card--${status}`
}
</script>

<style scoped>
.game-card {
  margin-bottom: 16px;
  transition: all 0.3s ease;
  border-radius: 8px;
  overflow: hidden;
}

.game-card:hover {
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  transform: translateY(-2px);
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 12px;
}

.title-section {
  flex: 1;
  min-width: 0;
}

.game-title {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
  color: #303133;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.game-description {
  margin: 4px 0 0 0;
  font-size: 12px;
  color: #909399;
  line-height: 1.4;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

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

.detail-button {
  width: 100%;
  border-radius: 4px;
}

/* 状态特定样式 */
.game-card--waiting {
  border-left: 4px solid #E6A23C;
}

.game-card--running {
  border-left: 4px solid #67C23A;
}

.game-card--paused {
  border-left: 4px solid #E6A23C;
}

.game-card--ended {
  border-left: 4px solid #606266;
}

.game-card--hidden {
  border-left: 4px solid #C0C4CC;
  opacity: 0.7;
}

.game-card--deleted {
  border-left: 4px solid #F56C6C;
  opacity: 0.7;
}
</style>