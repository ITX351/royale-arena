<template>
  <div class="actor-header">
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
        <el-button
          v-if="canPreviewRules"
          type="primary"
          link
          @click="showRulesPreview = true"
          :disabled="!game.rules_config"
          class="rules-preview-button"
        >
          浏览当前规则解析
        </el-button>
      </div>
    </el-card>

    <el-dialog
      v-model="showRulesPreview"
      title="当前规则解析"
      width="min(90vw, 800px)"
      destroy-on-close
    >
      <div class="rules-preview-dialog">
        <GameRulesPreview
          v-if="game.rules_config"
          :rules-config="game.rules_config"
          class="dialog-preview"
        />
        <el-empty v-else description="暂无规则配置" />
      </div>
      <template #footer>
        <el-button @click="showRulesPreview = false">关闭</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRouter } from 'vue-router'
import { ArrowDown, ArrowUp } from '@element-plus/icons-vue'
import type { GameWithRules } from '@/types/game'
import { GameStatus } from '@/types/game'
import { formatGameStatus, getStatusTagType } from '@/utils/gameUtils'
import GameRulesPreview from '@/components/GameRulesPreview.vue'

// Props
const props = defineProps<{
  game: GameWithRules
  actorPassword: string
}>()

// Router
const router = useRouter()

// 响应式状态
const showDetails = ref(false)
const showRulesPreview = ref(false)

// 计算属性
const statusDisplayText = computed(() => {
  return formatGameStatus(props.game.status)
})

const statusTagType = computed(() => {
  return getStatusTagType(props.game.status)
})

const canPreviewRules = computed(() => {
  return props.game.status === GameStatus.RUNNING || props.game.status === GameStatus.PAUSED
})

// 方法实现
const formatDate = (dateString: string) => {
  return new Date(dateString).toLocaleString('zh-CN')
}

const goHome = () => {
  router.push('/')
}
</script>

<style scoped>
@import '@/styles/shared-header.css';

.actor-header {
  margin-bottom: 24px;
}

.rules-preview-button {
  margin-top: 12px;
}

.rules-preview-dialog {
  max-height: 60vh;
  overflow-y: auto;
}

.dialog-preview {
  width: 100%;
}

.header-actions {
  display: flex;
  gap: 12px;
  flex-wrap: nowrap;
  flex-shrink: 0;
  overflow-x: auto;
  align-items: center;
}

.header-actions :deep(.el-button) {
  white-space: nowrap;
}
</style>