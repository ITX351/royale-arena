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
        
        <!-- 横向排列的控制面板 -->
        <div class="horizontal-controls">
          <!-- 天气控制 -->
          <el-card class="control-card" shadow="hover">
            <template #header>
              <div class="card-header">
                <h4>天气控制</h4>
              </div>
            </template>
            <div class="weather-control">
              <el-row :gutter="10" align="middle">
                <el-col :span="16">
                  <el-slider 
                    v-model="weatherValue" 
                    :min="0" 
                    :max="2" 
                    :step="0.1"
                    @change="handleWeatherChange"
                  />
                </el-col>
                <el-col :span="8">
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
              </el-row>
              <p>当前天气值: {{ weatherValue }}</p>
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
          
          <!-- 缩圈地点设置 -->
          <el-card class="control-card" shadow="hover">
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
        
        <!-- 地点状态管理和玩家状态管理卡片 -->
        <PlaceStatusCard 
          :places="placeList" 
          @place-status-change="handlePlaceStatusChange"
        />
        
        <PlayerStatusCard 
          :players="playerList" 
          @player-binding-change="handlePlayerBindingChange"
        />
        
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
import { ref, reactive, computed, watch } from 'vue'
import { ElMessage } from 'element-plus'
import { useGameStateStore } from '@/stores/gameState'
import type { GameWithRules } from '@/types/game'
import type { Player, Place } from '@/types/directorGameState'
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
  return store.placeList
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

.horizontal-controls {
  display: flex;
  gap: 20px;
  flex-wrap: wrap;
}

.control-card {
  flex: 1;
  min-width: 300px;
}

.weather-control {
  padding: 10px 0;
}

.night-time-control {
  padding: 10px 0;
}

.circle-places-control {
  padding: 10px 0;
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
</style>