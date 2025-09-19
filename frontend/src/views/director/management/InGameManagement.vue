<template>
  <div class="in-game-management">
    <el-card class="management-card">
      <template #header>
        <div class="card-header">
          <h3>游戏中管理</h3>
        </div>
      </template>
      
      <div class="management-content">
        <el-alert
          title="游戏中管理功能"
          type="info"
          show-icon
          :closable="false"
          class="management-info"
        >
          <template #default>
            <p>游戏正在进行中，您可以在此管理游戏进程。</p>
            <p>目前支持的操作包括暂停游戏和结束游戏。</p>
          </template>
        </el-alert>
        
        <!-- 空投设置面板 -->
        <AirdropPanel 
          :game-id="game.id"
          :places="placeList"
          @airdrop-sent="handleAirdropSent"
        />
        
        <!-- 广播消息面板 -->
        <BroadcastMessage 
          :game-id="game.id"
          :players="playerList"
          @message-sent="handleMessageSent"
        />
        
        <div class="management-actions">
          <el-button type="warning" size="large" @click="$emit('request-pause')">
            暂停游戏
          </el-button>
          <el-button type="danger" size="large" @click="$emit('request-end')">
            结束游戏
          </el-button>
        </div>
        
        <div class="management-note">
          <el-alert
            title="注意"
            type="warning"
            show-icon
            :closable="false"
          >
            <template #default>
              <p>游戏状态变更需要在导演控制台主界面进行操作。</p>
              <p>请使用页面顶部的状态控制按钮来管理游戏。</p>
            </template>
          </el-alert>
        </div>
      </div>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { ElMessage } from 'element-plus'
import type { GameWithRules } from '@/types/game'
import AirdropPanel from '../components/AirdropPanel.vue'
import BroadcastMessage from '../components/BroadcastMessage.vue'

// 定义组件属性
const props = defineProps<{
  game: GameWithRules
  directorPassword: string
}>()

// 定义事件发射
const emit = defineEmits<{
  (e: 'request-pause'): void
  (e: 'request-end'): void
}>()

// 计算属性
const playerList = computed(() => {
  return Object.values(props.game.players || {})
})

const placeList = computed(() => {
  return Object.values(props.game.rules_config?.map_config?.places || {})
})

// 方法实现
const handleAirdropSent = (items: string[], place: string) => {
  ElMessage.success(`空投已发送到地点: ${place}`)
  console.log('空投发送:', { items, place })
}

const handleMessageSent = (message: string, targetType: 'all' | 'player', targetPlayer?: string) => {
  if (targetType === 'all') {
    ElMessage.success('消息已广播给所有玩家')
  } else {
    const targetPlayerName = props.game.players?.[targetPlayer || '']?.name || targetPlayer
    ElMessage.success(`消息已发送给玩家: ${targetPlayerName}`)
  }
  console.log('消息发送:', { message, targetType, targetPlayer })
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
  gap: 24px;
}

.management-info {
  margin-bottom: 20px;
}

.management-actions {
  display: flex;
  justify-content: center;
  gap: 24px;
  flex-wrap: wrap;
}

.management-note {
  margin-top: auto;
}

@media (max-width: 768px) {
  .management-actions {
    flex-direction: column;
    align-items: center;
    gap: 16px;
  }
}
</style>