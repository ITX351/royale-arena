<template>
  <div class="in-game-state">
    <CompactActionPanel 
      v-if="player" 
      :player="player" 
      :places="placeList"
      :players="actorPlayerList"
      :global-state="globalState"
      :communication-visible="showInventoryDetails"
      @action="handlePlayerAction"
    />

    <el-alert 
      v-if="!player && gameStateStore.connecting" 
      title="正在连接到游戏服务器..." 
      type="info" 
      show-icon
      style="margin-bottom: 20px;"
    />

    <el-alert 
      v-else-if="!player" 
      title="暂无玩家数据，请确保已连接到游戏并有有效的玩家信息" 
      type="warning" 
      show-icon
      style="margin-bottom: 20px;"
    />

    <div class="main-content" v-if="player">
      <div class="inventory-section" :class="{ collapsed: !showInventoryDetails }">
        <div class="section-header">
          <h3>背包管理</h3>
          <el-button
            type="primary"
            link
            class="toggle-button"
            :icon="showInventoryDetails ? ArrowUp : ArrowDown"
            @click="toggleInventorySection"
          />
          <div class="section-controls">
            <el-tag v-if="player" type="info">总物品数: {{ totalItemCount }}</el-tag>
          </div>
        </div>
        <InventoryPanel
          v-if="showInventoryDetails"
          :player="player"
          :players="actorPlayerList"
          @equip-item="handleEquipItem"
          @use-item="handleUseItem"
          @discard-item="handleDiscardItem"
          @unequip-weapon="handleUnequipWeapon"
          @unequip-armor="handleUnequipArmor"
          @upgrade-equip="handleUpgradeEquip"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { useGameStateStore } from '@/stores/gameState'
import type { GameWithRules } from '@/types/game'
import type { Player, GlobalState, ActorPlayer, ActorPlace } from '@/types/gameStateTypes'
import { ArrowDown, ArrowUp } from '@element-plus/icons-vue'

import CompactActionPanel from '@/views/actor/components/CompactActionPanel.vue'
import InventoryPanel from '@/views/actor/components/InventoryPanel.vue'

defineProps<{
  game: GameWithRules | null
}>()

// 使用游戏状态存储
const gameStateStore = useGameStateStore()

const showInventoryDetails = ref(true)

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

const actorPlayerList = computed<ActorPlayer[]>(() => {
  // 从游戏状态存储中获取所有玩家列表（玩家视角）
  return gameStateStore.actorPlayerList
})

const totalItemCount = computed(() => {
  const current = player.value
  if (!current) return 0
  let count = current.inventory.length
  if (current.equipped_weapon) count++
  if (current.equipped_armor) count++
  return count
})

const toggleInventorySection = () => {
  showInventoryDetails.value = !showInventoryDetails.value
}

// 方法
const handlePlayerAction = (action: string, params: Record<string, any> = {}) => {
  // 通过WebSocket发送玩家操作
  gameStateStore.sendPlayerAction(action, params)
}

const handleEquipItem = (itemId: string) => {
  handlePlayerAction('equip', { item_id: itemId })
}

const handleUseItem = (payload: { itemId: string; params?: Record<string, any> }) => {
  const { itemId, params } = payload
  handlePlayerAction('use', {
    item_id: itemId,
    ...(params || {})
  })
}

const handleDiscardItem = (itemId: string) => {
  handlePlayerAction('throw', { item_id: itemId })
}

const handleUnequipWeapon = () => {
  handlePlayerAction('unequip', { slot_type: 'weapon' })
}

const handleUnequipArmor = () => {
  handlePlayerAction('unequip', { slot_type: 'armor' })
}

const handleUpgradeEquip = (payload: { itemId: string; slotType: 'weapon' | 'armor' }) => {
  handlePlayerAction('upgrade_equip', {
    item_id: payload.itemId,
    slot_type: payload.slotType
  })
}
</script>

<style scoped>
.in-game-state {
  display: flex;
  flex-direction: column;
  gap: 16px;
  padding: 16px 0;
  height: 100%;
  overflow-y: auto;
  width: 100%;
  align-items: stretch;
}

.main-content {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.inventory-section {
  background: #ffffff;
  border-radius: 8px;
  padding: 16px;
  border: 1px solid #e1e6f0;
  min-height: 300px;
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
  padding-bottom: 8px;
  border-bottom: 1px solid #f0f2f5;
  flex-wrap: nowrap;
}

.section-header h3 {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
  color: #303133;
  white-space: nowrap;
}

.section-header :deep(.el-tag) {
  white-space: nowrap;
}

.section-controls {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-left: auto;
}

.toggle-button {
  padding: 0;
}

.inventory-section.collapsed {
  min-height: auto;
  padding-bottom: 12px;
}

@media (max-width: 768px) {
  .in-game-state {
    padding: 12px 0;
    gap: 12px;
  }

  .inventory-section {
    padding: 12px;
    min-height: auto;
  }

  .section-header {
    gap: 8px;
  }
}

@media (max-width: 600px) {
  .in-game-state {
    padding: 8px 0;
  }
}

@media (min-width: 769px) and (max-width: 1024px) {
  .in-game-state {
    gap: 12px;
  }
}

@media (min-width: 1025px) {
  .in-game-state {
    gap: 16px;
  }
}
</style>