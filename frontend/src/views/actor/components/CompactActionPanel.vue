<template>
  <!-- 玩家状态栏 -->
  <div class="player-status-bar">
    <div class="status-item">
      <span class="status-value name">{{ player.name }}</span>
    </div>
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
    <div v-if="playerBleedDamage > 0" class="status-item bleed">
      <span class="status-label">流血:</span>
      <span class="status-value bleed">{{ playerBleedDamage }}</span>
    </div>
    <div class="status-item votes">
      <span class="status-label">票数:</span>
      <span class="status-value votes">{{ playerVoteCount }}</span>
    </div>
  </div>

  <!-- 核心操作区 -->
  <div class="core-actions">
    <div class="action-row">
      <!-- 出生/移动操作 -->
      <div class="action-group">
        <template v-if="!hasSpawned">
          <el-select 
            v-model="selectedPlace" 
            placeholder="选择出生地点" 
            size="small"
            style="width: 160px;"
          >
            <el-option
              v-for="place in places.filter(p => !p.is_destroyed)"
              :key="place.name"
              :label="place.name"
              :value="place.name"
            />
          </el-select>
          <el-button 
            type="primary" 
            size="small"
            :disabled="!selectedPlace || actionsDisabled"
            @click="handleBorn"
          >
            出生
          </el-button>
        </template>
        <template v-else>
          <el-select 
            v-model="targetPlace" 
            placeholder="选择目标地点" 
            size="small"
            style="width: 160px;"
          >
            <el-option
              v-for="place in places.filter(p => !p.is_destroyed && p.name !== player.location)"
              :key="place.name"
              :label="place.name"
              :value="place.name"
            />
          </el-select>
          <el-button 
            type="primary" 
            size="small"
            :disabled="!targetPlace || actionsDisabled"
            @click="handleMove"
          >
            移动
          </el-button>
        </template>
      </div>

      <div class="action-buttons" v-if="hasSpawned">
        <div class="primary-actions">
          <el-button 
            type="success" 
            size="small"
            :disabled="actionsDisabled"
            @click="handleSearch"
          >
            搜索
          </el-button>
          <el-button 
            type="danger" 
            size="small"
            :disabled="actionsDisabled || !hasValidTarget"
            @click="handleAttack"
          >
            攻击
          </el-button>
          <el-button 
            type="warning" 
            size="small"
            :disabled="actionsDisabled || !hasItemTarget"
            @click="handlePick"
          >
            捡拾
          </el-button>
        </div>
        <div class="rest-status-chip" :class="restStatusClass">
          <span class="rest-status-text">{{ restStatusLabel }}</span>
        </div>
      </div>
    </div>

    <div class="timing-hints">
      <span class="timing-text timing-search">
        <template v-if="player.is_bound">
          <span class="bound-warning">当前被捆绑，无法行动</span>
        </template>
        <template v-else>
          <template v-if="canSearchNow">
            <span class="search-ready">当前可以搜索</span>
          </template>
          <template v-else>
            <span class="search-pending">距离下一次搜索还有 {{ searchCooldownRemaining }} 秒</span>
          </template>
        </template>
      </span>
      <span class="timing-text timing-night">{{ nightCountdownMessage }}</span>
    </div>

    <div class="search-result-brief">
      <span class="search-result-text">{{ searchResultText }}</span>
    </div>
  </div>

  <!-- 通信快捷区 -->
  <div class="communication-actions">
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
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import type { Player, ActorPlayer,ActorPlace, GlobalState } from '@/types/gameStateTypes'
import { calculatePlayerVotes } from '@/utils/playerUtils'

const props = defineProps<{
  player: Player
  places: ActorPlace[]
  players: ActorPlayer[]
  globalState: GlobalState | null
}>()

const emit = defineEmits<{
  action: [action: string, params: Record<string, any>]
}>()

// 响应式数据
const selectedPlace = ref('')
const targetPlace = ref('')
const targetPlayer = ref('')
const deliverMessage = ref('')
const directorMessage = ref('')
const now = ref(Date.now())
let timer: number | null = null

// 计算属性
const hasValidTarget = computed(() => {
  return props.player.last_search_result?.target_type === 'player'
})

const hasItemTarget = computed(() => {
  return props.player.last_search_result?.target_type === 'item'
})

const hasSpawned = computed(() => {
  return Boolean(props.player.location)
})

const playerBleedDamage = computed(() => {
  const bleed = props.player.bleed_damage
  return typeof bleed === 'number' && bleed > 0 ? bleed : 0
})

const isResting = computed(() => {
  return Boolean(props.player.rest_mode)
})

const restStatusLabel = computed(() => {
  return isResting.value ? '静养中' : '已行动'
})

const restStatusClass = computed(() => {
  return isResting.value ? 'rest-active' : 'rest-inactive'
})

const playerVoteCount = computed(() => {
  return calculatePlayerVotes(props.player)
})

const nightStartMs = computed(() => {
  if (!props.globalState?.night_start_time) return null
  const value = new Date(props.globalState.night_start_time).getTime()
  return Number.isNaN(value) ? null : value
})

const nightEndMs = computed(() => {
  if (!props.globalState?.night_end_time) return null
  const value = new Date(props.globalState.night_end_time).getTime()
  return Number.isNaN(value) ? null : value
})

const nightActionActive = computed(() => {
  if (!nightStartMs.value || !nightEndMs.value) {
    return true
  }
  if (nightEndMs.value <= nightStartMs.value) {
    return now.value >= nightStartMs.value
  }
  return now.value >= nightStartMs.value && now.value <= nightEndMs.value
})

const actionsDisabled = computed(() => {
  return !nightActionActive.value || props.player.is_bound
})

const nightCountdownMessage = computed(() => {
  if (!nightStartMs.value) {
    return '夜晚行动时间未设置'
  }
  if (now.value < nightStartMs.value) {
    return `距离夜晚行动开始还有 ${formatDuration(nightStartMs.value - now.value)}`
  }
  if (!nightEndMs.value) {
    return '夜晚行动结束时间未设置'
  }
  if (now.value <= nightEndMs.value) {
    return `距离夜晚行动结束还有 ${formatDuration(nightEndMs.value - now.value)}`
  }
  return '当前不在夜晚行动时间'
})

const searchCooldownSeconds = computed(() => {
  const raw = props.globalState?.rules_config?.player?.search_cooldown
  if (typeof raw === 'number' && raw > 0) {
    return raw
  }
  return 0
})

const searchCooldownRemaining = computed(() => {
  if (!searchCooldownSeconds.value) {
    return '0.0'
  }
  if (!props.player.last_search_time) {
    return '0.0'
  }
  const lastSearch = new Date(props.player.last_search_time).getTime()
  if (Number.isNaN(lastSearch)) {
    return '0.0'
  }
  const nextAvailable = lastSearch + searchCooldownSeconds.value * 1000
  const remainingMs = nextAvailable - now.value
  if (remainingMs <= 0) {
    return '0.0'
  }
  return (remainingMs / 1000).toFixed(1)
})

const canSearchNow = computed(() => {
  if (!searchCooldownSeconds.value) {
    return true
  }
  if (!props.player.last_search_time) {
    return true
  }
  const lastSearch = new Date(props.player.last_search_time).getTime()
  if (Number.isNaN(lastSearch)) {
    return true
  }
  const nextAvailable = lastSearch + searchCooldownSeconds.value * 1000
  return now.value >= nextAvailable
})

const otherPlayers = computed((): ActorPlayer[] => {
  return props.players.filter(p => p.id !== props.player.id)
})

const searchResultText = computed(() => {
  const result = props.player.last_search_result
  if (!result) {
    return '暂无搜索结果'
  }
  const typeLabel = result.target_type === 'player' ? '玩家' : '道具'
  return `最近发现${typeLabel}: ${result.target_name}`
})

onMounted(() => {
  timer = window.setInterval(() => {
    now.value = Date.now()
  }, 100)
})

onUnmounted(() => {
  if (timer !== null) {
    window.clearInterval(timer)
    timer = null
  }
})

// 事件处理
const handleBorn = () => {
  if (actionsDisabled.value) {
    return
  }
  emit('action', 'born', { place_name: selectedPlace.value })
  selectedPlace.value = ''
}

const handleMove = () => {
  if (actionsDisabled.value) {
    return
  }
  emit('action', 'move', { target_place: targetPlace.value })
  targetPlace.value = ''
}

const handleSearch = () => {
  if (actionsDisabled.value) {
    return
  }
  emit('action', 'search', {})
}

const handleAttack = () => {
  if (actionsDisabled.value) {
    return
  }
  const target = props.player.last_search_result
  if (target?.target_type === 'player') {
    emit('action', 'attack', { target_player_id: target.target_id })
  } else {
    emit('action', 'attack', {})
  }
}

const handlePick = () => {
  if (actionsDisabled.value) {
    return
  }
  emit('action', 'pick', {})
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

function formatDuration(durationMs: number) {
  if (durationMs <= 0) {
    return '0秒'
  }
  const totalSeconds = Math.floor(durationMs / 1000)
  const minutes = Math.floor(totalSeconds / 60)
  const seconds = totalSeconds % 60
  if (minutes > 0) {
    return `${minutes}分${seconds}秒`
  }
  return `${seconds}秒`
}
</script>

<style scoped>
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

.status-value.name {
  color: #303133;
}

.status-item.bleed {
  background: #fde2e2;
  border-radius: 4px;
  padding: 4px 8px;
}

.status-value.bleed {
  color: #c45656;
}

.status-item.votes {
  margin-left: auto;
}

.status-value.votes {
  color: #303133;
}

.rest-status-chip {
  margin-left: auto;
  padding: 4px 10px;
  border-radius: 12px;
  font-size: 13px;
  display: flex;
  align-items: center;
}

.rest-status-chip.rest-active {
  background: #f0f9eb;
}

.rest-status-chip.rest-inactive {
  background: #f4f4f5;
}

.rest-status-text {
  color: #303133;
}

.rest-status-chip.rest-active .rest-status-text {
  color: #67c23a;
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

.action-buttons {
  flex: 1;
  display: flex;
  align-items: center;
  gap: 12px;
}

.primary-actions {
  display: flex;
  gap: 12px;
}

.primary-actions :deep(.el-button + .el-button) {
  margin-left: 0;
}

.timing-hints {
  display: flex;
  flex-direction: row;
  justify-content: space-between;
  align-items: center;
  gap: 12px;
  margin-top: 12px;
  color: #606266;
  width: 100%;
}

.timing-text {
  font-size: 12px;
}

.timing-search {
  flex: 1;
  min-width: 180px;
  display: flex;
  justify-content: center;
  text-align: center;
}

.timing-night {
  flex: 1;
  text-align: right;
  min-width: 180px;
}

.search-ready {
  color: #67c23a;
  font-weight: 700;
}

.search-pending {
  color: #f56c6c;
}

.bound-warning {
  color: #e6a23c;
  background-color: #fdf6ec;
  padding: 4px 8px;
  border-radius: 4px;
  font-weight: 700;
}

.search-result-brief {
  margin-top: 12px;
  padding: 8px 12px;
  border: 1px dashed #dcdfe6;
  border-radius: 6px;
  background: #ffffff;
}

.search-result-text {
  font-size: 13px;
  color: #303133;
}

.quick-item-row {
  display: flex;
  flex-direction: column;
  gap: 12px;
  align-items: stretch;
}

.equipped-weapons, .equipped-armors, .hand-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 8px 12px;
  background: #f0f2f5;
  border-radius: 4px;
}

.item-display {
  display: flex;
  gap: 8px;
  align-items: center;
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
  .player-status-bar {
    flex-direction: column;
    gap: 8px;
  }

  .status-item.votes {
    margin-left: 0;
    align-self: flex-end;
  }

  .rest-status-chip {
    margin-left: 0;
  }

  .primary-actions {
    justify-content: center;
    width: 100%;
  }
  
  .action-row {
    flex-direction: column;
    align-items: stretch;
  }
  
  .action-group {
    justify-content: center;
  }

  .action-buttons {
    justify-content: center;
    flex-wrap: wrap;
  }

  .timing-hints {
    flex-direction: column;
    align-items: stretch;
    gap: 8px;
  }

  .timing-search,
  .timing-night {
    text-align: left;
    min-width: auto;
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