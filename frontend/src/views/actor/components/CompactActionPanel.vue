<template>
  <div class="compact-action-panel">
    <!-- 玩家状态栏 -->
    <div class="player-status-bar">
      <div class="status-item">
        <span class="status-label">生命:</span>
        <span class="status-value life">{{ player.life }}</span>
      </div>
      <div class="status-item">
        <span class="status-label">体力:</span>
        <span class="status-value strength">{{ player.strength }}</span>
      </div>
      <div class="status-item">
        <span class="status-label">位置:</span>
        <span class="status-value location">{{ player.location || '未出生' }}</span>
      </div>
    </div>

    <!-- 核心操作区 -->
    <div class="core-actions">
      <h4 class="section-title">核心操作</h4>
      <div class="action-row">
        <!-- 出生/移动操作 -->
        <div class="action-group">
          <template v-if="!player.location">
            <!-- 出生操作 -->
            <el-select 
              v-model="selectedPlace" 
              placeholder="选择出生地点" 
              size="small"
              style="width: 160px;"
            >
              <el-option
                v-for="place in places"
                :key="place.name"
                :label="place.name"
                :value="place.name"
                :disabled="place.is_destroyed"
              />
            </el-select>
            <el-button 
              type="primary" 
              size="small"
              :disabled="!selectedPlace"
              @click="handleBorn"
            >
              出生
            </el-button>
          </template>
          <template v-else>
            <!-- 移动操作 -->
            <el-select 
              v-model="targetPlace" 
              placeholder="选择目标地点" 
              size="small"
              style="width: 160px;"
            >
              <el-option
                v-for="place in places"
                :key="place.name"
                :label="place.name"
                :value="place.name"
                :disabled="place.is_destroyed || place.name === player.location"
              />
            </el-select>
            <el-button 
              type="primary" 
              size="small"
              :disabled="!targetPlace"
              @click="handleMove"
            >
              移动
            </el-button>
          </template>
        </div>

        <!-- 搜索操作 -->
        <el-button 
          type="success" 
          size="small"
          @click="handleSearch"
        >
          搜索
        </el-button>

        <!-- 攻击操作 -->
        <el-button 
          type="danger" 
          size="small"
          :disabled="!hasValidTarget"
          @click="handleAttack"
        >
          攻击
        </el-button>

        <!-- 捡拾操作 -->
        <el-button 
          type="warning" 
          size="small"
          :disabled="!hasItemTarget"
          @click="handlePick"
        >
          捡拾
        </el-button>
      </div>
    </div>

    <!-- 道具快操区 -->
    <div class="item-quick-actions" v-if="player.inventory.length > 0">
      <h4 class="section-title">道具快操</h4>
      <div class="quick-item-row">
        <!-- 当前装备 -->
        <div class="equipped-item" v-if="equippedItem">
          <span class="item-label">装备:</span>
          <span class="item-name">{{ equippedItem.name }}</span>
          <el-button 
            size="small" 
            type="text"
            @click="handleUnequip"
          >
            卸下
          </el-button>
        </div>

        <!-- 当前手持 -->
        <div class="hand-item" v-if="handItem">
          <span class="item-label">手持:</span>
          <span class="item-name">{{ handItem.name }}</span>
          <el-button 
            size="small" 
            type="text"
            @click="handleUseItem"
          >
            使用
          </el-button>
          <el-button 
            size="small" 
            type="text"
            @click="handlePutDown"
          >
            放下
          </el-button>
        </div>

        <!-- 背包道具选择 -->
        <div class="inventory-selector">
          <el-select 
            v-model="selectedItem" 
            placeholder="选择道具" 
            size="small"
            style="width: 140px;"
          >
            <el-option
              v-for="item in player.inventory"
              :key="item.id"
              :label="item.name"
              :value="item.id"
            />
          </el-select>
          <el-button 
            size="small"
            :disabled="!selectedItem"
            @click="handleEquipSelected"
          >
            装备
          </el-button>
          <el-button 
            size="small"
            :disabled="!selectedItem"
            @click="handleDiscardSelected"
          >
            丢弃
          </el-button>
        </div>
      </div>
    </div>

    <!-- 通信快捷区 -->
    <div class="communication-actions">
      <h4 class="section-title">通信</h4>
      <div class="comm-row">
        <!-- 传音 -->
        <div class="deliver-group">
          <el-select 
            v-model="targetPlayer" 
            placeholder="选择玩家" 
            size="small"
            style="width: 120px;"
          >
            <el-option
              v-for="otherPlayer in otherPlayers"
              :key="otherPlayer.id"
              :label="otherPlayer.name"
              :value="otherPlayer.id"
            />
          </el-select>
          <el-input 
            v-model="deliverMessage" 
            placeholder="传音内容"
            size="small"
            style="width: 150px;"
            @keyup.enter="handleDeliver"
          />
          <el-button 
            size="small"
            :disabled="!targetPlayer || !deliverMessage"
            @click="handleDeliver"
          >
            传音
          </el-button>
        </div>

        <!-- 发送给导演 -->
        <div class="director-message-group">
          <el-input 
            v-model="directorMessage" 
            placeholder="发送给导演"
            size="small"
            style="width: 200px;"
            @keyup.enter="handleSendToDirector"
          />
          <el-button 
            size="small"
            :disabled="!directorMessage"
            @click="handleSendToDirector"
          >
            发送
          </el-button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import type { Player, ActorPlace } from '@/types/gameStateTypes'

const props = defineProps<{
  player: Player
  places: ActorPlace[]
}>()

const emit = defineEmits<{
  action: [action: string, params: Record<string, any>]
}>()

// 响应式数据
const selectedPlace = ref('')
const targetPlace = ref('')
const selectedItem = ref('')
const targetPlayer = ref('')
const deliverMessage = ref('')
const directorMessage = ref('')

// 计算属性
const equippedItem = computed(() => {
  if (!props.player.equipped_item) return null
  return props.player.inventory.find(item => item.id === props.player.equipped_item)
})

const handItem = computed(() => {
  if (!props.player.hand_item) return null
  return props.player.inventory.find(item => item.id === props.player.hand_item)
})

const hasValidTarget = computed(() => {
  return props.player.last_search_result?.target_type === 'player'
})

const hasItemTarget = computed(() => {
  return props.player.last_search_result?.target_type === 'item'
})

const otherPlayers = computed(() => {
  // 这里需要从全局状态获取其他玩家列表
  // 暂时返回空数组，实际使用时需要从父组件传入或从store获取
  return []
})

// 事件处理
const handleBorn = () => {
  emit('action', 'born', { place_name: selectedPlace.value })
  selectedPlace.value = ''
}

const handleMove = () => {
  emit('action', 'move', { target_place: targetPlace.value })
  targetPlace.value = ''
}

const handleSearch = () => {
  emit('action', 'search')
}

const handleAttack = () => {
  emit('action', 'attack')
}

const handlePick = () => {
  emit('action', 'pick')
}

const handleEquipSelected = () => {
  emit('action', 'equip', { item_id: selectedItem.value })
  selectedItem.value = ''
}

const handleDiscardSelected = () => {
  emit('action', 'throw', { item_id: selectedItem.value })
  selectedItem.value = ''
}

const handleUseItem = () => {
  if (handItem.value) {
    emit('action', 'use', { item_id: handItem.value.id })
  }
}

const handlePutDown = () => {
  emit('action', 'put_down')
}

const handleUnequip = () => {
  // 卸下装备的逻辑需要后端支持，暂时不实现
  console.log('卸下装备功能待实现')
}

const handleDeliver = () => {
  emit('action', 'deliver', { 
    target_player_id: targetPlayer.value, 
    message: deliverMessage.value 
  })
  deliverMessage.value = ''
}

const handleSendToDirector = () => {
  emit('action', 'send', { message: directorMessage.value })
  directorMessage.value = ''
}
</script>

<style scoped>
.compact-action-panel {
  padding: 16px;
  background: #f8f9fa;
  border-radius: 8px;
  margin-bottom: 16px;
}

.player-status-bar {
  display: flex;
  gap: 24px;
  padding: 12px 16px;
  background: #ffffff;
  border-radius: 6px;
  margin-bottom: 16px;
  border: 1px solid #e1e6f0;
}

.status-item {
  display: flex;
  align-items: center;
  gap: 8px;
}

.status-label {
  font-size: 14px;
  color: #606266;
  font-weight: 500;
}

.status-value {
  font-size: 16px;
  font-weight: bold;
}

.status-value.life {
  color: #f56c6c;
}

.status-value.strength {
  color: #67c23a;
}

.status-value.location {
  color: #409eff;
}

.section-title {
  margin: 0 0 12px 0;
  font-size: 14px;
  font-weight: 600;
  color: #303133;
}

.core-actions, .item-quick-actions, .communication-actions {
  margin-bottom: 16px;
}

.action-row {
  display: flex;
  gap: 12px;
  align-items: center;
  flex-wrap: wrap;
}

.action-group {
  display: flex;
  gap: 8px;
  align-items: center;
}

.quick-item-row {
  display: flex;
  gap: 16px;
  align-items: center;
  flex-wrap: wrap;
}

.equipped-item, .hand-item {
  display: flex;
  gap: 8px;
  align-items: center;
  padding: 8px 12px;
  background: #f0f2f5;
  border-radius: 4px;
}

.item-label {
  font-size: 12px;
  color: #909399;
}

.item-name {
  font-size: 14px;
  font-weight: 500;
  color: #303133;
}

.inventory-selector {
  display: flex;
  gap: 8px;
  align-items: center;
}

.comm-row {
  display: flex;
  gap: 16px;
  align-items: center;
  flex-wrap: wrap;
}

.deliver-group, .director-message-group {
  display: flex;
  gap: 8px;
  align-items: center;
}

/* 响应式设计 */
@media (max-width: 768px) {
  .compact-action-panel {
    padding: 12px;
  }
  
  .player-status-bar {
    flex-direction: column;
    gap: 8px;
  }
  
  .action-row {
    flex-direction: column;
    align-items: stretch;
  }
  
  .action-group {
    justify-content: center;
  }
  
  .quick-item-row, .comm-row {
    flex-direction: column;
    align-items: stretch;
  }
}

@media (max-width: 600px) {
  .deliver-group, .director-message-group {
    flex-direction: column;
    width: 100%;
  }
  
  .el-select, .el-input {
    width: 100% !important;
  }
}
</style>