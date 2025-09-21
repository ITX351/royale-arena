<template>
  <el-card class="compact-action-panel">
    <template #header>
      <div class="card-header">
        <h3>操作面板</h3>
      </div>
    </template>

    <div class="action-grid">
      <!-- 出生操作 -->
      <el-button 
        v-if="showBornAction" 
        type="success" 
        round 
        class="action-button born-button"
        @click="handleBorn"
        :disabled="!canPerformAction"
      >
        出生
      </el-button>

      <!-- 移动操作 -->
      <el-button 
        type="primary" 
        round 
        class="action-button move-button"
        @click="handleMove"
        :disabled="!canPerformAction"
      >
        移动
      </el-button>

      <!-- 搜索操作 -->
      <el-button 
        type="warning" 
        round 
        class="action-button search-button"
        @click="handleSearch"
        :disabled="!canPerformAction"
      >
        搜索
      </el-button>

      <!-- 捡拾操作 -->
      <el-button 
        v-if="showPickAction" 
        type="success" 
        round 
        class="action-button pick-button"
        @click="handlePick"
        :disabled="!canPerformAction"
      >
        捡拾
      </el-button>

      <!-- 攻击操作 -->
      <el-button 
        v-if="showAttackAction" 
        type="danger" 
        round 
        class="action-button attack-button"
        @click="handleAttack"
        :disabled="!canPerformAction"
      >
        攻击
      </el-button>

      <!-- 装备操作 -->
      <el-button 
        v-if="showEquipAction" 
        type="info" 
        round 
        class="action-button equip-button"
        @click="handleEquip"
        :disabled="!canPerformAction"
      >
        装备
      </el-button>

      <!-- 使用道具操作 -->
      <el-button 
        v-if="showUseAction" 
        type="info" 
        round 
        class="action-button use-button"
        @click="handleUse"
        :disabled="!canPerformAction"
      >
        使用
      </el-button>

      <!-- 丢弃道具操作 -->
      <el-button 
        v-if="showThrowAction" 
        type="info" 
        round 
        class="action-button throw-button"
        @click="handleThrow"
        :disabled="!canPerformAction"
      >
        丢弃
      </el-button>

      <!-- 传音操作 -->
      <el-button 
        type="success" 
        round 
        class="action-button deliver-button"
        @click="handleDeliver"
        :disabled="!canPerformAction"
      >
        传音
      </el-button>

      <!-- 发送消息给导演操作 -->
      <el-button 
        type="success" 
        round 
        class="action-button send-button"
        @click="handleSendToDirector"
        :disabled="!canPerformAction"
      >
        发送消息
      </el-button>
    </div>

    <!-- 出生地点选择对话框 -->
    <el-dialog 
      v-model="bornDialogVisible" 
      title="选择出生地点" 
      width="300px"
      class="action-dialog"
    >
      <el-select 
        v-model="selectedPlace" 
        placeholder="请选择出生地点" 
        style="width: 100%"
      >
        <el-option
          v-for="place in availablePlaces"
          :key="place.name"
          :label="place.name"
          :value="place.name"
        />
      </el-select>
      <template #footer>
        <span class="dialog-footer">
          <el-button @click="bornDialogVisible = false">取消</el-button>
          <el-button type="primary" @click="confirmBorn">确定</el-button>
        </span>
      </template>
    </el-dialog>

    <!-- 移动地点选择对话框 -->
    <el-dialog 
      v-model="moveDialogVisible" 
      title="选择移动地点" 
      width="300px"
      class="action-dialog"
    >
      <el-select 
        v-model="selectedPlace" 
        placeholder="请选择移动地点" 
        style="width: 100%"
      >
        <el-option
          v-for="place in availablePlaces"
          :key="place.name"
          :label="place.name"
          :value="place.name"
        />
      </el-select>
      <template #footer>
        <span class="dialog-footer">
          <el-button @click="moveDialogVisible = false">取消</el-button>
          <el-button type="primary" @click="confirmMove">确定</el-button>
        </span>
      </template>
    </el-dialog>

    <!-- 传音目标选择对话框 -->
    <el-dialog 
      v-model="deliverDialogVisible" 
      title="选择传音目标" 
      width="300px"
      class="action-dialog"
    >
      <el-select 
        v-model="selectedPlayer" 
        placeholder="请选择玩家" 
        style="width: 100%"
      >
        <el-option
          v-for="player in otherPlayers"
          :key="player.id"
          :label="player.name"
          :value="player.id"
        />
      </el-select>
      <el-input
        v-model="deliverMessage"
        placeholder="请输入消息内容"
        type="textarea"
        :rows="3"
        style="margin-top: 15px"
      />
      <template #footer>
        <span class="dialog-footer">
          <el-button @click="deliverDialogVisible = false">取消</el-button>
          <el-button type="primary" @click="confirmDeliver">发送</el-button>
        </span>
      </template>
    </el-dialog>

    <!-- 发送消息给导演对话框 -->
    <el-dialog 
      v-model="sendDialogVisible" 
      title="发送消息给导演" 
      width="300px"
      class="action-dialog"
    >
      <el-input
        v-model="sendMessage"
        placeholder="请输入消息内容"
        type="textarea"
        :rows="3"
      />
      <template #footer>
        <span class="dialog-footer">
          <el-button @click="sendDialogVisible = false">取消</el-button>
          <el-button type="primary" @click="confirmSendToDirector">发送</el-button>
        </span>
      </template>
    </el-dialog>
  </el-card>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { ElMessage } from 'element-plus'
import type { Player, ActorPlace as Place } from '@/types/gameStateTypes'
import { webSocketService } from '@/services/webSocketService'

const props = defineProps<{
  player: Player | null
  places: Record<string, Place> | null
}>()

const emit = defineEmits<{
  (e: 'action', action: string, params: Record<string, any>): void
}>()

// 响应式数据
const bornDialogVisible = ref(false)
const moveDialogVisible = ref(false)
const deliverDialogVisible = ref(false)
const sendDialogVisible = ref(false)
const selectedPlace = ref('')
const selectedPlayer = ref('')
const deliverMessage = ref('')
const sendMessage = ref('')

// 计算属性
const canPerformAction = computed(() => {
  return props.player !== null && props.player.is_alive
})

const showBornAction = computed(() => {
  return props.player !== null && !props.player.location
})

const showPickAction = computed(() => {
  return props.player !== null && 
         props.player.last_search_result !== null && 
         props.player.last_search_result.target_type === 'item'
})

const showAttackAction = computed(() => {
  return props.player !== null && 
         props.player.last_search_result !== null && 
         props.player.last_search_result.target_type === 'player'
})

const showEquipAction = computed(() => {
  // 简化实现，实际应检查玩家是否有可装备的物品
  return props.player !== null && props.player.inventory.length > 0
})

const showUseAction = computed(() => {
  // 简化实现，实际应检查玩家是否持有可使用的物品
  return props.player !== null && props.player.hand_item !== null
})

const showThrowAction = computed(() => {
  // 简化实现，实际应检查玩家是否有可丢弃的物品
  return props.player !== null && props.player.inventory.length > 0
})

const availablePlaces = computed(() => {
  if (!props.places) return []
  return Object.values(props.places).filter(place => !place.is_destroyed)
})

const otherPlayers = computed(() => {
  if (!props.player || !props.places) return []
  
  // 获取玩家当前位置的其他玩家
  const currentPlace = props.places[props.player.location]
  if (!currentPlace) return []
  
  return currentPlace.players
    .filter(playerId => playerId !== props.player?.id)
    .map(playerId => {
      // 这里简化处理，实际应该从玩家列表中获取玩家信息
      return {
        id: playerId,
        name: `玩家${playerId.substring(0, 4)}`
      }
    })
})

// 方法
const handleBorn = () => {
  if (!canPerformAction.value) return
  
  // 如果只有一个可选地点，直接出生
  if (availablePlaces.value.length === 1) {
    emit('action', 'born', { place_name: availablePlaces.value[0].name })
  } else {
    // 否则显示选择对话框
    selectedPlace.value = ''
    bornDialogVisible.value = true
  }
}

const handleMove = () => {
  if (!canPerformAction.value) return
  
  // 显示移动地点选择对话框
  selectedPlace.value = ''
  moveDialogVisible.value = true
}

const handleSearch = () => {
  if (!canPerformAction.value) return
  
  // 发送搜索操作
  emit('action', 'search', {})
}

const handlePick = () => {
  if (!canPerformAction.value || !showPickAction.value) return
  
  // 发送捡拾操作
  emit('action', 'pick', {})
}

const handleAttack = () => {
  if (!canPerformAction.value || !showAttackAction.value) return
  
  // 发送攻击操作
  emit('action', 'attack', {})
}

const handleEquip = () => {
  if (!canPerformAction.value) return
  
  // 这里应该显示装备物品选择对话框，简化处理直接发送事件
  ElMessage.info('请选择要装备的物品')
  emit('action', 'equip', {})
}

const handleUse = () => {
  if (!canPerformAction.value) return
  
  // 发送使用道具操作
  emit('action', 'use', {})
}

const handleThrow = () => {
  if (!canPerformAction.value) return
  
  // 这里应该显示丢弃物品选择对话框，简化处理直接发送事件
  ElMessage.info('请选择要丢弃的物品')
  emit('action', 'throw', {})
}

const handleDeliver = () => {
  if (!canPerformAction.value) return
  
  // 显示传音目标选择对话框
  selectedPlayer.value = ''
  deliverMessage.value = ''
  deliverDialogVisible.value = true
}

const handleSendToDirector = () => {
  if (!canPerformAction.value) return
  
  // 显示发送消息给导演对话框
  sendMessage.value = ''
  sendDialogVisible.value = true
}

const confirmBorn = () => {
  if (!selectedPlace.value) {
    ElMessage.warning('请选择出生地点')
    return
  }
  
  emit('action', 'born', { place_name: selectedPlace.value })
  bornDialogVisible.value = false
}

const confirmMove = () => {
  if (!selectedPlace.value) {
    ElMessage.warning('请选择移动地点')
    return
  }
  
  emit('action', 'move', { target_place: selectedPlace.value })
  moveDialogVisible.value = false
}

const confirmDeliver = () => {
  if (!selectedPlayer.value) {
    ElMessage.warning('请选择传音目标')
    return
  }
  
  if (!deliverMessage.value.trim()) {
    ElMessage.warning('请输入消息内容')
    return
  }
  
  emit('action', 'deliver', { 
    target_player_id: selectedPlayer.value, 
    message: deliverMessage.value 
  })
  deliverDialogVisible.value = false
}

const confirmSendToDirector = () => {
  if (!sendMessage.value.trim()) {
    ElMessage.warning('请输入消息内容')
    return
  }
  
  emit('action', 'send', { message: sendMessage.value })
  sendDialogVisible.value = false
}
</script>

<style scoped>
.compact-action-panel {
  background-color: #fff9f0;
  border: 1px solid #f0d6b3;
}

.compact-action-panel :deep(.el-card__header) {
  background-color: #fdf1e0;
  padding: 10px 15px;
}

.card-header h3 {
  margin: 0;
  color: #333;
  font-size: 16px;
}

.action-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 10px;
  padding: 10px 0;
}

.action-button {
  height: 50px;
  font-size: 14px;
  font-weight: 500;
  border: none;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
  transition: all 0.2s;
}

.action-button:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 8px rgba(0, 0, 0, 0.15);
}

.action-button:active {
  transform: translateY(0);
}

.action-button:disabled {
  opacity: 0.6;
  transform: none;
  box-shadow: none;
}

.born-button {
  background: linear-gradient(135deg, #67c23a 0%, #49a72a 100%);
  color: white;
}

.move-button {
  background: linear-gradient(135deg, #409eff 0%, #2a7fdf 100%);
  color: white;
}

.search-button {
  background: linear-gradient(135deg, #e6a23c 0%, #c98a2a 100%);
  color: white;
}

.pick-button {
  background: linear-gradient(135deg, #67c23a 0%, #49a72a 100%);
  color: white;
}

.attack-button {
  background: linear-gradient(135deg, #f56c6c 0%, #d74a4a 100%);
  color: white;
}

.equip-button,
.use-button,
.throw-button {
  background: linear-gradient(135deg, #909399 0%, #73767a 100%);
  color: white;
}

.deliver-button,
.send-button {
  background: linear-gradient(135deg, #67c23a 0%, #49a72a 100%);
  color: white;
}

.action-dialog :deep(.el-dialog__header) {
  background-color: #fdf1e0;
  padding: 15px;
}

.action-dialog :deep(.el-dialog__title) {
  color: #333;
  font-size: 16px;
  font-weight: 500;
}

.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
  margin-top: 15px;
}

@media (max-width: 768px) {
  .action-grid {
    grid-template-columns: repeat(2, 1fr);
  }
  
  .action-button {
    font-size: 13px;
    height: 45px;
  }
}

@media (max-width: 480px) {
  .action-grid {
    grid-template-columns: 1fr;
  }
  
  .action-button {
    font-size: 14px;
    height: 50px;
  }
}
</style>