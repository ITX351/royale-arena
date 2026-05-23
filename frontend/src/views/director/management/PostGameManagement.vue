<template>
  <div class="post-game-management">
    <el-card class="management-card">
      <template #header>
        <div class="card-header">
          <h3>游戏结束</h3>
        </div>
      </template>
      
      <div class="management-content">
        <el-alert
          title="游戏已结束"
          type="info"
          show-icon
          :closable="false"
          class="management-info"
        >
          <template #default>
            <p>当前游戏已经结束，无法进行进一步的管理操作。</p>
            <p>您可以查看游戏结果或重新创建新的游戏。</p>
          </template>
        </el-alert>
        
        <div class="management-message">
          <el-icon size="64" color="#909399"><InfoFilled /></el-icon>
          <p>游戏状态：{{ statusText }}</p>
        </div>
        
        <div class="management-actions">
          <el-button type="primary" @click="goToGameDetail">
            查看游戏详情
          </el-button>
          <el-button @click="goToHome">
            返回首页
          </el-button>
        </div>
      </div>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useRouter } from 'vue-router'
import { InfoFilled } from '@element-plus/icons-vue'
import type { GameWithRules } from '@/types/game'
import { GameStatus } from '@/types/game'

// Props
const props = defineProps<{
  game: GameWithRules
}>()

const router = useRouter()

// 计算属性
const statusText = computed(() => {
  const statusMap: Record<string, string> = {
    [GameStatus.ENDED]: '已结束',
    [GameStatus.HIDDEN]: '已隐藏',
    [GameStatus.DELETED]: '已删除'
  }
  return statusMap[props.game.status] || '已结束'
})

// 方法实现
const goToGameDetail = () => {
  router.push(`/game/${props.game.id}`)
}

const goToHome = () => {
  router.push('/')
}
</script>

<style scoped>
.management-card {
  margin-bottom: 24px;
}

.card-header h3 {
  margin: 0;
  color: #303133;
}

.management-content {
  min-height: 300px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 24px;
  text-align: center;
}

.management-info {
  width: 100%;
}

.management-message {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 16px;
}

.management-message p {
  margin: 0;
  color: #909399;
  font-size: 16px;
}

.management-actions {
  display: flex;
  gap: 16px;
  flex-wrap: wrap;
  justify-content: center;
}

@media (max-width: 768px) {
  .management-content {
    padding: 20px 0;
  }
  
  .management-actions {
    flex-direction: column;
    width: 100%;
    max-width: 300px;
  }
  
  .management-actions .el-button {
    width: 100%;
  }
}
</style>