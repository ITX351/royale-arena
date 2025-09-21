<template>
  <el-card class="player-status-card">
    <template #header>
      <div class="card-header">
        <h3>玩家状态管理</h3>
      </div>
    </template>
    <div class="player-status-content">
      <el-table :data="playerList" style="width: 100%" size="small" max-height="200">
        <el-table-column prop="name" label="玩家" />
        <el-table-column label="操作">
          <template #default="scope">
            <el-button 
              size="small" 
              :type="scope.row.is_bound ? 'warning' : 'primary'"
              @click="togglePlayerBinding(scope.row.id)"
            >
              {{ scope.row.is_bound ? '松绑' : '捆绑' }}
            </el-button>
          </template>
        </el-table-column>
      </el-table>
    </div>
  </el-card>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useGameStateStore } from '@/stores/gameState'
import type { Player } from '@/types/gameStateTypes'

// 定义组件属性
const props = defineProps<{
  players: Player[]
}>()

// 定义事件发射
const emit = defineEmits<{
  (e: 'player-binding-change', playerId: string): void
}>()

const store = useGameStateStore()

// 计算属性
const playerList = computed<Player[]>(() => {
  return props.players
})

// 玩家状态管理方法
const togglePlayerBinding = (playerId: string) => {
  // 调用store中的方法处理玩家捆绑/松绑
  store.togglePlayerBinding(playerId)
  // 发送事件通知父组件
  emit('player-binding-change', playerId)
}
</script>

<style scoped>
.player-status-card {
  margin-bottom: 20px;
}

.card-header h3 {
  margin: 0;
  color: #303133;
}

.player-status-content {
  padding: 10px 0;
}
</style>