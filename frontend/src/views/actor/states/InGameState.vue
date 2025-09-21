<template>
  <div class="in-game-state">
    <el-card class="game-state-card">
      <template #header>
        <div class="card-header">
          <h3>游戏状态</h3>
        </div>
      </template>
      
      <!-- 玩家状态显示组件 -->
      <PlayerStatusCard 
        v-if="player" 
        :player="player" 
        :global-state="globalState"
      />

      <!-- 紧凑操作面板 -->
      <CompactActionPanel 
        v-if="player" 
        :player="player" 
        :places="placeList"
        @action="handlePlayerAction"
      />

      <!-- 连接状态提示 -->
      <el-alert 
        v-if="!player && gameStateStore.connecting" 
        title="正在连接到游戏服务器..." 
        type="info" 
        show-icon
        style="margin-bottom: 20px;"
      />

      <!-- 无玩家数据提示 -->
      <el-alert 
        v-else-if="!player" 
        title="暂无玩家数据，请确保已连接到游戏并有有效的玩家信息" 
        type="warning" 
        show-icon
        style="margin-bottom: 20px;"
      />

      <el-row :gutter="20" class="main-content">
        <!-- 左侧：背包面板 -->
        <el-col :span="24" :md="12">
          <InventoryPanel
            :player="player"
            @equip-item="handleEquipItem"
            @use-item="handleUseItem"
            @discard-item="handleDiscardItem"
            @put-down-hand-item="handlePutDownHandItem"
          />
        </el-col>

        <!-- 右侧：搜索结果展示 -->
        <el-col :span="24" :md="12">
          <SearchResultDisplay
            :player="player"
            @search="handleSearch"
            @pick="handlePick"
            @attack="handleAttack"
          />
        </el-col>
      </el-row>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted } from 'vue'
import { useGameStateStore } from '@/stores/gameState'
import type { GameWithRules } from '@/types/game'
import type { Player, GlobalState, ActorPlace } from '@/types/gameStateTypes'
import { GameStatus } from '@/types/game'
import { formatTimestamp, formatGameStatus } from '@/utils/gameUtils'
import { webSocketService } from '@/services/webSocketService'

// 导入组件
import PlayerStatusCard from '@/views/actor/components/PlayerStatusCard.vue'
import CompactActionPanel from '@/views/actor/components/CompactActionPanel.vue'
import InventoryPanel from '@/views/actor/components/InventoryPanel.vue'
import SearchResultDisplay from '@/views/actor/components/SearchResultDisplay.vue'

const props = defineProps<{
  game: GameWithRules | null
}>()

// 使用游戏状态存储
const gameStateStore = useGameStateStore()

// 计算属性
const player = computed<Player | null>(() => {
  // 从游戏状态存储中获取当前玩家信息（玩家视角）
  return gameStateStore.actorPlayer
})

const globalState = computed<GlobalState | null>(() => {
  // 从游戏状态存储中获取全局状态信息
  return gameStateStore.globalState
})

const placeList = computed<ActorPlace[]>(() => {
  // 从游戏状态存储中获取地点列表（玩家视角）
  return gameStateStore.actorPlaceList
})

// 组件挂载时的操作
onMounted(() => {
  // 可以在这里添加组件挂载时需要执行的逻辑
})

// 组件卸载时的操作
onUnmounted(() => {
  // 可以在这里添加组件卸载时需要执行的逻辑
})

// 方法
const handlePlayerAction = (action: string, params: Record<string, any> = {}) => {
  // 通过WebSocket发送玩家操作
  gameStateStore.sendPlayerAction(action, params)
}

const handleEquipItem = (itemId: string) => {
  handlePlayerAction('equip', { item_id: itemId })
}

const handleUseItem = (itemId: string) => {
  handlePlayerAction('use', { item_id: itemId })
}

const handleDiscardItem = (itemId: string) => {
  handlePlayerAction('throw', { item_id: itemId })
}

const handlePutDownHandItem = () => {
  handlePlayerAction('put_down')
}

const handleSearch = () => {
  handlePlayerAction('search')
}

const handlePick = () => {
  handlePlayerAction('pick')
}

const handleAttack = (targetPlayerId: string) => {
  handlePlayerAction('attack', { target_player_id: targetPlayerId })
}

// 出生操作
const handleBorn = (placeName: string) => {
  handlePlayerAction('born', { place_name: placeName })
}

// 移动操作
const handleMove = (targetPlace: string) => {
  handlePlayerAction('move', { target_place: targetPlace })
}

// 传音操作
const handleDeliver = (targetPlayerId: string, message: string) => {
  handlePlayerAction('deliver', { target_player_id: targetPlayerId, message })
}

// 发送消息给导演操作
const handleSend = (message: string) => {
  handlePlayerAction('send', { message })
}
</script>

<style scoped>
.in-game-state {
  display: flex;
  flex-direction: column;
  gap: 20px;
  padding: 20px;
  height: 100%;
  overflow-y: auto;
  max-width: 100%;
  margin: 0 auto;
}

.game-state-card {
  height: fit-content;
}

.card-header h3 {
  margin: 0;
  color: #303133;
}

.main-content {
  flex: 1;
}

/* 响应式设计 */
@media (max-width: 768px) {
  .in-game-state {
    padding: 10px;
    gap: 10px;
  }
  
  .el-row {
    gap: 10px;
  }
}

@media (min-width: 769px) and (max-width: 1024px) {
  .in-game-state {
    max-width: 600px;
  }
}

@media (min-width: 1025px) {
  .in-game-state {
    max-width: 720px;
  }
}
</style>