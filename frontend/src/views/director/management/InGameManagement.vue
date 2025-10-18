<template>
  <div class="in-game-management">
    <el-card class="management-card">
      <template #header>
        <div class="card-header">
          <h3>游戏中管理</h3>
        </div>
      </template>
      
      <div class="management-content">
        <div class="settings-panel">
          <div class="setting-row">
            <div class="setting-label">天气值</div>
            <el-input
              v-model="weatherText"
              placeholder="0 - 2"
              size="small"
              style="width: 120px"
            />
            <el-button type="primary" size="small" @click="applyWeather">更新天气</el-button>
          </div>

          <div class="setting-row">
            <div class="setting-label">夜晚时间</div>
            <el-date-picker
              v-model="nightTimeForm.startTime"
              type="datetime"
              placeholder="开始时间"
              format="YYYY-MM-DD HH:mm"
              value-format="YYYY-MM-DDTHH:mm:ssZ"
              clearable
              size="small"
            />
            <el-date-picker
              v-model="nightTimeForm.endTime"
              type="datetime"
              placeholder="结束时间"
              format="YYYY-MM-DD HH:mm"
              value-format="YYYY-MM-DDTHH:mm:ssZ"
              clearable
              size="small"
            />
            <el-button type="primary" size="small" @click="setNightTime">设置</el-button>
            <el-button size="small" @click="clearNightTime">清空</el-button>
          </div>

          <div class="setting-row">
            <div class="setting-label">缩圈地点</div>
            <el-select
              v-model="selectedDestroyPlaces"
              multiple
              placeholder="请选择缩圈地点"
              size="small"
              style="flex: 1"
            >
              <el-option
                v-for="place in placeList"
                :key="place.name"
                :label="place.name"
                :value="place.name"
              />
            </el-select>
            <el-button type="primary" size="small" @click="setDestroyPlaces">设置</el-button>
          </div>
        </div>

        <!-- 地点状态管理和玩家状态管理卡片 - 修改为横跨整个屏幕 -->
        <div class="full-width-section">
          <PlaceStatusCard 
            :places="placeList" 
            @place-status-change="handlePlaceStatusChange"
          />
        </div>
        
        <div class="full-width-section">
          <PlayerStatusCard 
            :players="playerList" 
            @player-binding-change="handlePlayerBindingChange"
          />
        </div>
        
        <!-- 空投设置面板 -->
        <AirdropPanel 
          :game-id="game.id"
          @airdrop-accepted="handleAirdropAccepted"
        />
        
        <!-- 广播消息面板 -->
        <BroadcastMessage 
          ref="broadcastMessageRef"
          :game-id="game.id"
          :players="playerList"
          @message-sent="handleMessageSent"
        />
      </div>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, watch } from 'vue'
import { ElMessage } from 'element-plus'
import { useGameStateStore } from '@/stores/gameState'
import type { GameWithRules } from '@/types/game'
import type { Player, DirectorPlace as Place } from '@/types/gameStateTypes'
import PlaceStatusCard from '../components/PlaceStatusCard.vue'
import PlayerStatusCard from '../components/PlayerStatusCard.vue'
import AirdropPanel from '../components/AirdropPanel.vue'
import BroadcastMessage from '../components/BroadcastMessage.vue'

// 定义组件属性
defineProps<{
  game: GameWithRules
  directorPassword: string
}>()

// 定义事件发射
const emit = defineEmits<{
  (e: 'request-pause'): void
  (e: 'request-end'): void
}>()

// 定义暴露给父组件的方法
defineExpose({
  setBroadcastTarget
})

const store = useGameStateStore()

// 添加BroadcastMessage组件引用
const broadcastMessageRef = ref<any>(null)

// 天气控制相关
const weatherText = ref('1.0')

// 夜晚时间表单
const nightTimeForm = reactive({
  startTime: null as string | null,
  endTime: null as string | null
})

// 缩圈地点选择
const selectedDestroyPlaces = ref<string[]>([])

// 计算属性
const playerList = computed<Player[]>(() => {
  return store.playerList
})

const placeList = computed<Place[]>(() => {
  return store.directorPlaceList
})

// 监听全局状态变化，更新控制面板值
watch(
  () => store.globalState,
  (newState) => {
    if (newState) {
      weatherText.value = formatWeather(newState.weather)
      nightTimeForm.startTime = newState.night_start_time
      nightTimeForm.endTime = newState.night_end_time
      selectedDestroyPlaces.value = newState.next_night_destroyed_places || []
    }
  },
  { immediate: true }
)

// 天气控制方法
const applyWeather = () => {
  const numeric = Number(weatherText.value)
  if (Number.isNaN(numeric)) {
    ElMessage.error('请输入有效的天气数值')
    return
  }
  if (numeric < 0 || numeric > 2) {
    ElMessage.error('天气值必须在0-2之间')
    weatherText.value = clampWeather(numeric)
    return
  }
  const rounded = Number(numeric.toFixed(1))
  weatherText.value = rounded.toFixed(1)
  store.updateWeather(rounded)
  ElMessage.success(`天气已更新为: ${numeric}`)
}

// 夜晚时间设置方法
const setNightTime = () => {
  // 调用store中的方法设置夜晚时间
  store.setNightTime(nightTimeForm.startTime, nightTimeForm.endTime)
  ElMessage.success('夜晚时间设置已发送')
}

const clearNightTime = () => {
  nightTimeForm.startTime = null
  nightTimeForm.endTime = null
  // 调用store中的方法清空夜晚时间
  store.setNightTime(null, null)
  ElMessage.success('夜晚时间已清空')
}

// 缩圈地点设置方法
const setDestroyPlaces = () => {
  // 调用store中的方法设置缩圈地点
  store.setDestroyPlaces(selectedDestroyPlaces.value)
  ElMessage.success('缩圈地点设置已发送')
}

// 地点状态调整方法
const handlePlaceStatusChange = (placeName: string) => {
  ElMessage.success(`地点 "${placeName}" 状态已更新`)
}

// 玩家状态管理方法
const handlePlayerBindingChange = (playerId: string) => {
  // 获取玩家信息用于消息提示
  const player = store.directorPlayers[playerId]
  if (player) {
    ElMessage.success(`玩家 "${player.name}" 状态已更新`)
  }
}

// 新增方法：设置广播目标玩家
function setBroadcastTarget(playerId: string) {
  // 检查BroadcastMessage组件引用是否存在
  if (broadcastMessageRef.value) {
    // 如果面板是折叠的，先展开它
    if (typeof broadcastMessageRef.value.expandPanel === 'function') {
      broadcastMessageRef.value.expandPanel();
    }
    
    // 调用BroadcastMessage组件中的方法来设置目标玩家
    broadcastMessageRef.value.setTargetPlayer(playerId);
    // 聚焦到消息输入框
    broadcastMessageRef.value.focusMessageInput();
  }
}

// 空投和广播消息相关方法
const handleAirdropAccepted = (items: any[], place: string) => {
  ElMessage.success(`空投已发送到地点: ${place}`)
  console.log('空投发送:', { items, place })
}

const handleMessageSent = (message: string, targetType: 'all' | 'player', targetPlayer?: string) => {
  console.log('消息发送:', { message, targetType, targetPlayer })
}

function formatWeather(value?: number) {
  if (typeof value !== 'number' || Number.isNaN(value)) {
    return '1.0'
  }
  return value.toFixed(1)
}

function clampWeather(value: number) {
  const clamped = Math.min(2, Math.max(0, value))
  return clamped.toFixed(1)
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

.card-header h4 {
  margin: 0;
  color: #606266;
  font-size: 16px;
  text-align: center; /* 居中标题文本 */
  font-weight: 600; /* 稍微加粗标题 */
}

.management-content {
  min-height: 300px;
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.full-width-section {
  width: 100%;
}

/* 调整PlaceStatusCard和PlayerStatusCard样式 */
.full-width-section :deep(.el-card) {
  width: 100%;
}

.settings-panel {
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 12px 0;
}

.setting-row {
  display: flex;
  align-items: center;
  gap: 12px;
}

.setting-label {
  width: 72px;
  font-size: 13px;
  color: #606266;
  flex-shrink: 0;
}

@media (max-width: 768px) {
  .setting-row {
    flex-direction: column;
    align-items: stretch;
  }

  .setting-label {
    width: 100%;
  }
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
</style>