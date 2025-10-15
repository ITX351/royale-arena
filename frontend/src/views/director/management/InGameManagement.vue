<template>
  <div class="in-game-management">
    <el-card class="management-card">
      <template #header>
        <div class="card-header">
          <h3>游戏中管理</h3>
        </div>
      </template>
      
      <div class="management-content">
        <!-- 横向排列的控制面板 -->
        <div class="control-section">
          <!-- 第一行：天气控制和夜晚时间设置并排 -->
          <div class="horizontal-controls">
            <!-- 天气控制 -->
            <el-card class="control-card" shadow="hover">
              <template #header>
                <div class="card-header">
                  <h4>天气控制</h4>
                </div>
              </template>
              <div class="weather-control">
                <!-- 滑块单独一行 -->
                <div class="slider-container">
                  <el-slider 
                    v-model="weatherValue" 
                    :min="0" 
                    :max="2" 
                    :step="0.1"
                    @change="handleWeatherChange"
                  />
                </div>
                
                <!-- 输入框和文字在同一行 -->
                <el-row :gutter="10" align="middle" class="value-row">
                  <el-col :span="11">
                    <el-input-number 
                      v-model="weatherValue" 
                      :min="0" 
                      :max="2" 
                      :step="0.1"
                      :precision="1"
                      @change="handleWeatherChange"
                      style="width: 100%"
                      size="small"
                    />
                  </el-col>
                  <el-col :span="13">
                    <div class="weather-value-text">
                      当前天气值: {{ weatherValue }}
                    </div>
                  </el-col>
                </el-row>
              </div>
            </el-card>
            
            <!-- 夜晚时间设置 -->
            <el-card class="control-card" shadow="hover">
              <template #header>
                <div class="card-header">
                  <h4>夜晚时间设置</h4>
                </div>
              </template>
              <div class="night-time-control">
                <el-form :model="nightTimeForm" label-width="80px" size="small">
                  <el-form-item label="开始时间">
                    <el-date-picker
                      v-model="nightTimeForm.startTime"
                      type="datetime"
                      placeholder="选择开始时间"
                      format="YYYY-MM-DD HH:mm"
                      value-format="YYYY-MM-DDTHH:mm:ssZ"
                      clearable
                      style="width: 100%"
                    />
                  </el-form-item>
                  <el-form-item label="结束时间">
                    <el-date-picker
                      v-model="nightTimeForm.endTime"
                      type="datetime"
                      placeholder="选择结束时间"
                      format="YYYY-MM-DD HH:mm"
                      value-format="YYYY-MM-DDTHH:mm:ssZ"
                      clearable
                      style="width: 100%"
                    />
                  </el-form-item>
                  <el-form-item>
                    <el-button type="primary" @click="setNightTime" size="small">设置时间</el-button>
                    <el-button @click="clearNightTime" size="small">清空时间</el-button>
                  </el-form-item>
                </el-form>
              </div>
            </el-card>
          </div>
          
          <!-- 第二行：下一轮缩圈位置独占一行 -->
          <div class="full-width-control">
            <el-card class="control-card full-width-card" shadow="hover">
              <template #header>
                <div class="card-header">
                  <h4>下一轮缩圈位置</h4>
                </div>
              </template>
              <div class="circle-places-control">
                <el-select
                  v-model="selectedDestroyPlaces"
                  multiple
                  placeholder="请选择缩圈地点"
                  style="width: 100%"
                  size="small"
                >
                  <el-option
                    v-for="place in placeList"
                    :key="place.name"
                    :label="place.name"
                    :value="place.name"
                  />
                </el-select>
                <div style="margin-top: 10px">
                  <el-button type="primary" @click="setDestroyPlaces" size="small">设置缩圈地点</el-button>
                </div>
              </div>
            </el-card>
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
const props = defineProps<{
  game: GameWithRules
  directorPassword: string
}>()

// 定义事件发射
const emit = defineEmits<{
  (e: 'request-pause'): void
  (e: 'request-end'): void
}>()

const store = useGameStateStore()

// 天气控制相关
const weatherValue = ref<number>(1.0)

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
      weatherValue.value = newState.weather || 1.0
      nightTimeForm.startTime = newState.night_start_time
      nightTimeForm.endTime = newState.night_end_time
      selectedDestroyPlaces.value = newState.next_night_destroyed_places || []
    }
  },
  { immediate: true }
)

// 天气控制方法
const handleWeatherChange = (value: number | undefined) => {
  if (value === undefined) return
  
  // 验证输入值
  if (value < 0 || value > 2) {
    ElMessage.error('天气值必须在0-2之间')
    // 重置为有效值
    weatherValue.value = Math.min(2, Math.max(0, value))
    return
  }
  
  // 调用store中的方法更新天气
  store.updateWeather(value)
  ElMessage.success(`天气已更新为: ${value}`)
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
const handlePlaceStatusChange = (placeName: string, isDestroyed: boolean) => {
  ElMessage.success(`地点 "${placeName}" 状态已更新`)
}

// 玩家状态管理方法
const handlePlayerBindingChange = (playerId: string) => {
  // 获取玩家信息用于消息提示
  const player = store.players[playerId]
  if (player) {
    ElMessage.success(`玩家 "${player.name}" 状态已更新`)
  }
}

// 空投和广播消息相关方法
const handleAirdropAccepted = (items: any[], place: string) => {
  ElMessage.success(`空投已发送到地点: ${place}`)
  console.log('空投发送:', { items, place })
}

const handleMessageSent = (message: string, targetType: 'all' | 'player', targetPlayer?: string) => {
  if (targetType === 'all') {
    ElMessage.success('消息已广播给所有玩家')
  } else {
    // 获取玩家信息
    const players = store.players
    const targetPlayerName = targetPlayer && players[targetPlayer] ? players[targetPlayer].name : targetPlayer
    ElMessage.success(`消息已发送给玩家: ${targetPlayerName || '未知'}`)
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

.management-info {
  margin-bottom: 10px;
}

/* 控制面板区域 */
.control-section {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

/* 第一行控制面板 - 天气和夜晚时间并排 */
.horizontal-controls {
  display: flex;
  gap: 20px;
  flex-wrap: wrap;
  justify-content: space-between;
}

.control-card {
  flex: 1;
  min-width: 250px; /* 减小最小宽度以适应并排显示 */
}

.full-width-card {
  width: 100%;
}

.weather-control {
  padding: 5px 10px; /* 调整内边距 */
}

.slider-container {
  margin-bottom: 10px; /* 滑块底部间距 */
}

.slider-container .el-slider {
  width: 100%;
}

.value-row {
  flex-wrap: nowrap;
}

.weather-value-text {
  font-size: 14px;
  color: #606266;
  white-space: nowrap; /* 防止文字换行 */
  overflow: hidden;
  text-overflow: ellipsis;
}

.night-time-control {
  padding: 10px 0;
}

.circle-places-control {
  padding: 10px 0;
}

.circle-places-control .el-select {
  width: 100%;
}

.circle-places-control .el-button {
  width: 100%;
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

/* 新增样式 - 全宽部分 */
.full-width-section {
  width: 100%;
}

/* 调整PlaceStatusCard和PlayerStatusCard样式 */
.full-width-section :deep(.el-card) {
  width: 100%;
}

@media (max-width: 768px) {
  .horizontal-controls {
    flex-direction: column;
  }
  
  .control-card {
    min-width: 100%;
  }
  
  .management-actions {
    flex-direction: column;
    align-items: center;
    gap: 16px;
  }
}

/* 中等屏幕设备优化 */
@media (min-width: 769px) and (max-width: 1024px) {
  .control-card {
    min-width: 200px;
  }
}

/* 大屏幕设备优化 */
@media (min-width: 1025px) {
  .control-card {
    min-width: 250px;
  }
}
</style>