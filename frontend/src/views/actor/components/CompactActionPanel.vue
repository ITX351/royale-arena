<template>
  <!-- 玩家状态栏 -->
  <div class="player-status-bar">
    <div class="status-item">
      <span class="status-value name">{{ player.name }}</span>
    </div>
    <div class="status-item">
      <span class="status-label">生命:</span>
      <span :class="['status-value', 'life', lifeAnimationClass]">{{ player.life }}</span>
    </div>
    <div class="status-item">
      <span class="status-label">体力:</span>
      <span class="status-value strength">{{ player.strength }}</span>
    </div>
    <div class="status-item">
      <span class="status-label">位置:</span>
      <span class="status-value location">{{ locationDisplay }}</span>
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
            style="width: 120px;"
            placement="bottom-start"
            :popper-options="selectPopperOptions"
            filterable
            :class="{ 'safe-zone-selected': isSafePlace(selectedPlace) }"
          >
            <el-option
              v-for="place in availablePlaces"
              :key="place.name"
              :label="place.name"
              :value="place.name"
              :class="{ 'safe-zone-option': isSafePlace(place.name) }"
              :style="getPlaceOptionStyle(place.name)"
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
            style="width: 120px;"
            placement="bottom-start"
            :popper-options="selectPopperOptions"
            filterable
            :class="{ 'safe-zone-selected': isSafePlace(targetPlace) }"
          >
            <el-option
              v-for="place in availablePlaces"
              :key="place.name"
              :label="place.name"
              :value="place.name"
              :disabled="place.name === player.location"
              :class="{ 'safe-zone-option': isSafePlace(place.name) }"
              :style="getPlaceOptionStyle(place.name)"
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
        <div class="rest-status-chip rest-desktop" :class="restStatusClass">
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
  <div class="communication-actions" v-if="props.communicationVisible">
    <div class="comm-row">
      <!-- 传音 -->
      <div class="deliver-group">
        <el-select 
          v-model="targetPlayer" 
          placeholder="选择玩家" 
          size="small"
          style="width: 120px;"
          filterable
        >
          <el-option
            v-for="otherPlayer in sortedOtherPlayers"
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
          :maxlength="MESSAGE_MAX_LENGTH"
          show-word-limit
          @keyup.enter="handleDeliver"
        />
        <el-button 
          size="small"
          :disabled="!targetPlayer || !deliverMessage.trim() || deliverMessageTooLong"
          @click="handleDeliver"
        >
          传音
        </el-button>
        <span v-if="deliverMessageTooLong" class="input-error">内容不能超过 {{ MESSAGE_MAX_LENGTH }} 字</span>
      </div>

      <!-- 发送给导演 -->
      <div class="director-message-group">
        <el-input 
          v-model="directorMessage" 
          placeholder="发送给导演"
          size="small"
          style="width: 200px;"
          :maxlength="MESSAGE_MAX_LENGTH"
          show-word-limit
          @keyup.enter="handleSendToDirector"
        />
        <el-button 
          size="small"
          :disabled="!directorMessage.trim() || directorMessageTooLong"
          @click="handleSendToDirector"
        >
          发送
        </el-button>
        <span v-if="directorMessageTooLong" class="input-error">内容不能超过 {{ MESSAGE_MAX_LENGTH }} 字</span>
      </div>
      <div
        class="rest-status-chip rest-mobile"
        :class="restStatusClass"
      >
        <span class="rest-status-text">{{ restStatusLabel }}</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from 'vue'
import { storeToRefs } from 'pinia'
import type { Player, ActorPlayer,ActorPlace, GlobalState } from '@/types/gameStateTypes'
import { calculatePlayerVotes } from '@/utils/playerUtils'
import { useGameStateStore } from '@/stores/gameState'

const props = withDefaults(defineProps<{
  player: Player
  places: ActorPlace[]
  players: ActorPlayer[]
  globalState: GlobalState | null
  communicationVisible?: boolean
}>(), {
  communicationVisible: true
})

const emit = defineEmits<{
  action: [action: string, params: Record<string, any>]
}>()

// 响应式数据
const selectedPlace = ref('')
const targetPlace = ref('')
const targetPlayer = ref('')
const deliverMessage = ref('')
const directorMessage = ref('')
const gameStateStore = useGameStateStore()
const { serverOffsetMs } = storeToRefs(gameStateStore)
const now = ref(Date.now() + serverOffsetMs.value)
let timer: number | null = null
const lifeAnimation = ref<'damage' | 'heal' | ''>('')
let lifeAnimationTimer: number | null = null
const MESSAGE_MAX_LENGTH = 100

const selectPopperOptions = {
  modifiers: [
    {
      name: 'flip',
      options: {
        fallbackPlacements: []
      }
    }
  ]
}

const safeZoneOptionStyle: Record<string, string> = {
  color: '#67c23a',
  fontWeight: '600'
}

const safePlaceNames = computed<Set<string>>(() => {
  const safePlaces = props.globalState?.rules_config?.map?.safe_places
  if (Array.isArray(safePlaces)) {
    return new Set<string>(safePlaces)
  }
  return new Set<string>()
})

const comparePlaceName = (a: ActorPlace, b: ActorPlace) => {
  const localeResult = a.name.localeCompare(b.name, 'zh-CN-u-co-pinyin')
  return localeResult || a.name.localeCompare(b.name)
}

const availablePlaces = computed<ActorPlace[]>(() => {
  const validPlaces = props.places.filter(place => !place.is_destroyed)
  if (!validPlaces.length) {
    return []
  }

  const safe: ActorPlace[] = []
  const regular: ActorPlace[] = []

  for (const place of validPlaces) {
    if (safePlaceNames.value.has(place.name)) {
      safe.push(place)
    } else {
      regular.push(place)
    }
  }

  safe.sort(comparePlaceName)
  regular.sort(comparePlaceName)

  return [...safe, ...regular]
})

const isSafePlace = (placeName: string) => {
  return safePlaceNames.value.has(placeName)
}

const getPlaceOptionStyle = (placeName: string): Record<string, string> | undefined => {
  return isSafePlace(placeName) ? safeZoneOptionStyle : undefined
}

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

const locationDisplay = computed(() => {
  if (!props.player.is_alive) {
    return '已死亡'
  }
  return props.player.location || '未出生'
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

const sortedOtherPlayers = computed(() => {
  return [...otherPlayers.value].sort((a, b) => {
    const localeResult = a.name.localeCompare(b.name, 'zh-CN-u-co-pinyin')
    return localeResult || a.name.localeCompare(b.name)
  })
})

const searchResultText = computed(() => {
  const result = props.player.last_search_result
  if (!result) {
    return '暂无搜索结果'
  }
  const typeLabel = result.target_type === 'player' ? '玩家' : '道具'
  return `最近发现${typeLabel}: ${result.target_name}`
})

const lifeAnimationClass = computed(() => {
  if (!lifeAnimation.value) {
    return ''
  }
  return lifeAnimation.value === 'damage' ? 'life-damage' : 'life-heal'
})

const deliverMessageTooLong = computed(() => {
  return deliverMessage.value.length > MESSAGE_MAX_LENGTH
})

const directorMessageTooLong = computed(() => {
  return directorMessage.value.length > MESSAGE_MAX_LENGTH
})

onMounted(() => {
  timer = window.setInterval(() => {
    now.value = Date.now() + serverOffsetMs.value
  }, 100)
})

watch(serverOffsetMs, (newOffset) => {
  now.value = Date.now() + newOffset
})

onUnmounted(() => {
  if (timer !== null) {
    window.clearInterval(timer)
    timer = null
  }
  if (lifeAnimationTimer !== null) {
    window.clearTimeout(lifeAnimationTimer)
    lifeAnimationTimer = null
  }
})

// Resetting the class allows repeated life changes to retrigger the CSS animation.
const triggerLifeAnimation = (type: 'damage' | 'heal') => {
  if (lifeAnimationTimer !== null) {
    window.clearTimeout(lifeAnimationTimer)
    lifeAnimationTimer = null
  }

  lifeAnimation.value = ''

  window.requestAnimationFrame(() => {
    lifeAnimation.value = type
    lifeAnimationTimer = window.setTimeout(() => {
      lifeAnimation.value = ''
      lifeAnimationTimer = null
    }, 700)
  })
}

watch(
  () => props.player.life,
  (newLife, oldLife) => {
    if (typeof oldLife !== 'number') {
      return
    }

    if (newLife < oldLife) {
      triggerLifeAnimation('damage')
    } else if (newLife > oldLife) {
      triggerLifeAnimation('heal')
    }
  }
)

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
  emit('action', 'attack', {})
}

const handlePick = () => {
  if (actionsDisabled.value) {
    return
  }
  emit('action', 'pick', {})
}

const handleDeliver = () => {
  const trimmedMessage = deliverMessage.value.trim()
  if (!targetPlayer.value || !trimmedMessage || trimmedMessage.length > MESSAGE_MAX_LENGTH) {
    return
  }
  emit('action', 'deliver', {
    target_player_id: targetPlayer.value,
    message: trimmedMessage
  })
  deliverMessage.value = ''
}

const handleSendToDirector = () => {
  const trimmedMessage = directorMessage.value.trim()
  if (!trimmedMessage || trimmedMessage.length > MESSAGE_MAX_LENGTH) {
    return
  }
  emit('action', 'send', { message: trimmedMessage })
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

:deep(.el-select-dropdown__item.safe-zone-option) {
  color: #67c23a;
  font-weight: 600;
}

:deep(.el-select.safe-zone-selected .el-select__selected-item) {
  color: #67c23a;
  font-weight: 600;
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

.status-value.life.life-damage {
  animation: lifeShake 0.6s ease;
}

.status-value.life.life-heal {
  color: #409f4e;
  text-shadow: 0 0 6px rgba(103, 194, 58, 0.7);
  animation: lifeHeal 0.8s ease;
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

.rest-status-chip.rest-mobile {
  display: none;
  white-space: nowrap;
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

@keyframes lifeShake {
  0% {
    transform: translateX(0);
  }
  20% {
    transform: translateX(-4px);
  }
  40% {
    transform: translateX(4px);
  }
  60% {
    transform: translateX(-3px);
  }
  80% {
    transform: translateX(3px);
  }
  100% {
    transform: translateX(0);
  }
}

@keyframes lifeHeal {
  0% {
    transform: scale(1);
    opacity: 1;
  }
  40% {
    transform: scale(1.12);
    opacity: 0.9;
  }
  70% {
    transform: scale(1.05);
    opacity: 1;
  }
  100% {
    transform: scale(1);
    opacity: 1;
  }
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
  text-align: center;
}

.search-result-text {
  font-size: 13px;
  color: #303133;
  display: inline-block;
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

.input-error {
  color: #f56c6c;
  font-size: 12px;
  white-space: nowrap;
}

@media (min-width: 769px) {
  .comm-row {
    width: 100%;
  }

  .director-message-group {
    margin-left: auto;
  }
}

/* 响应式设计 */
@media (max-width: 768px) {
  .player-status-bar {
    flex-wrap: wrap;
    justify-content: left;
    gap: 12px;
  }

  .status-item {
    flex: 0 1 auto;
  }

  .status-item.votes {
    margin-left: 0;
  }

  .rest-status-chip {
    margin-left: 0;
  }

  .rest-status-chip.rest-desktop {
    display: none;
  }

  .rest-status-chip.rest-mobile {
    display: flex;
    margin-left: auto;
  }

  .primary-actions {
    justify-content: center;
    width: auto;
  }
  
  .action-row {
    flex-wrap: wrap;
    justify-content: left;
    align-items: left;
  }
  
  .action-group {
    justify-content: center;
  }

  .action-buttons {
    justify-content: center;
    flex-wrap: wrap;
  }

  .timing-hints {
    flex-direction: row;
    flex-wrap: wrap;
    justify-content: left;
    align-items: left;
    gap: 8px;
  }

  .timing-search,
  .timing-night {
    text-align: left;
    min-width: 150px;
  }
  
  .quick-item-row, .comm-row {
    flex-direction: row;
    flex-wrap: wrap;
    justify-content: left;
  }
}

@media (max-width: 600px) {
  .deliver-group, .director-message-group {
    flex-direction: row;
    flex-wrap: wrap;
    width: auto;
    justify-content: left;
  }
  
  .deliver-group .el-select,
  .deliver-group .el-input,
  .director-message-group .el-input {
    flex: 1 1 150px;
    width: auto !important;
  }

  .deliver-group .el-button,
  .director-message-group .el-button {
    flex: 0 0 auto;
  }

  .rest-status-chip.rest-mobile {
    flex: 0 0 auto;
  }
}
</style>