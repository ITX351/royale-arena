<template>
  <el-card class="player-status-card" :class="{ 'night-mode': isNight }">
    <template #header>
      <div class="card-header">
        <h3>玩家状态</h3>
        <div class="time-display">
          <span v-if="nextPhaseTime">{{ formattedCountdown }}</span>
        </div>
      </div>
    </template>

    <div class="status-content">
      <!-- 生命值 -->
      <div class="status-item">
        <span class="label">生命值</span>
        <el-progress 
          :percentage="lifePercentage" 
          :stroke-width="12" 
          :color="lifeColor" 
          :show-text="false"
        />
        <span class="value">{{ playerLife }}/100</span>
      </div>

      <!-- 体力值 -->
      <div class="status-item">
        <span class="label">体力值</span>
        <el-progress 
          :percentage="strengthPercentage" 
          :stroke-width="12" 
          :color="strengthColor" 
          :show-text="false"
        />
        <span class="value">{{ playerStrength }}/100</span>
      </div>

      <!-- 位置信息 -->
      <div class="status-item">
        <span class="label">当前位置</span>
        <span class="value">{{ playerLocation }}</span>
      </div>

      <!-- 下一阶段时间 -->
      <div class="status-item">
        <span class="label">下一阶段</span>
        <span class="value">{{ nextPhaseLabel }}</span>
      </div>

      <!-- 折叠的全局信息 -->
      <el-collapse v-model="activeCollapse">
        <el-collapse-item name="global-info">
          <template #title>
            <el-icon><ArrowDown /></el-icon>
            <span>全局信息</span>
          </template>
          <div class="global-info">
            <div class="info-item">
              <span class="label">天气</span>
              <span class="value">{{ weatherLabel }}</span>
            </div>
            <div class="info-item">
              <span class="label">游戏阶段</span>
              <span class="value">{{ gamePhaseLabel }}</span>
            </div>
            <div class="info-item">
              <span class="label">夜晚结束时间</span>
              <span class="value">{{ nightEndTime || '未设置' }}</span>
            </div>
            <div class="info-item">
              <span class="label">下一轮缩圈位置</span>
              <span class="value">{{ nextDestroyPlaces }}</span>
            </div>
          </div>
        </el-collapse-item>
      </el-collapse>
    </div>
  </el-card>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { ArrowDown } from '@element-plus/icons-vue'
import type { Player, GlobalState } from '@/types/gameStateTypes'

const props = defineProps<{
  player: Player | null
  globalState: GlobalState | null
}>()

// 响应式数据
const activeCollapse = ref('')
const countdown = ref<number | null>(null)
const countdownInterval = ref<number | null>(null)

// 计算属性
const playerLife = computed(() => props.player?.life || 0)
const playerStrength = computed(() => props.player?.strength || 0)
const playerLocation = computed(() => props.player?.location || '未知')

const lifePercentage = computed(() => Math.max(0, Math.min(100, playerLife.value)))
const strengthPercentage = computed(() => Math.max(0, Math.min(100, playerStrength.value)))

const lifeColor = computed(() => {
  if (lifePercentage.value > 70) return '#67c23a'
  if (lifePercentage.value > 30) return '#e6a23c'
  return '#f56c6c'
})

const strengthColor = computed(() => {
  if (strengthPercentage.value > 70) return '#409eff'
  if (strengthPercentage.value > 30) return '#e6a23c'
  return '#f56c6c'
})

const isNight = computed(() => props.globalState?.game_phase === 'night')

const weatherLabel = computed(() => {
  const weather = props.globalState?.weather || 0
  switch (weather) {
    case 0: return '晴天'
    case 1: return '雨天'
    case 2: return '暴风雨'
    default: return '未知'
  }
})

const gamePhaseLabel = computed(() => {
  const phase = props.globalState?.game_phase || 'day'
  return phase === 'night' ? '夜晚' : '白天'
})

const nightEndTime = computed(() => props.globalState?.night_end_time || null)

const nextDestroyPlaces = computed(() => {
  const places = props.globalState?.next_night_destroyed_places || []
  return places.length > 0 ? places.join(', ') : '未设置'
})

const nextPhaseTime = computed(() => {
  if (isNight.value) {
    return nightEndTime.value
  }
  return props.globalState?.night_start_time || null
})

const nextPhaseLabel = computed(() => {
  if (isNight.value) {
    return nightEndTime.value ? `夜晚结束 ${nightEndTime.value}` : '夜晚结束时间未设置'
  }
  return props.globalState?.night_start_time ? `夜晚开始 ${props.globalState.night_start_time}` : '夜晚开始时间未设置'
})

const formattedCountdown = computed(() => {
  if (countdown.value === null) return ''
  const minutes = Math.floor(countdown.value / 60)
  const seconds = countdown.value % 60
  return `${minutes.toString().padStart(2, '0')}:${seconds.toString().padStart(2, '0')}`
})

// 方法
const calculateCountdown = () => {
  if (!nextPhaseTime.value) {
    countdown.value = null
    return
  }

  try {
    // 将时间字符串转换为Date对象
    const targetTime = new Date(nextPhaseTime.value)
    const now = new Date()
    const diffSeconds = Math.floor((targetTime.getTime() - now.getTime()) / 1000)
    
    // 如果距离下一阶段开始时间小于5分钟(300秒)，显示倒计时
    if (diffSeconds > 0 && diffSeconds <= 300) {
      countdown.value = diffSeconds
    } else {
      countdown.value = null
    }
  } catch (error) {
    console.error('计算倒计时出错:', error)
    countdown.value = null
  }
}

// 生命周期
onMounted(() => {
  calculateCountdown()
  // 每秒更新一次倒计时
  countdownInterval.value = window.setInterval(() => {
    calculateCountdown()
  }, 1000)
})

onUnmounted(() => {
  if (countdownInterval.value) {
    clearInterval(countdownInterval.value)
  }
})
</script>

<style scoped>
.player-status-card {
  background: linear-gradient(135deg, #f5f7fa 0%, #e4e7f4 100%);
  border: 1px solid #dcdfe6;
}

.player-status-card.night-mode {
  background: linear-gradient(135deg, #2c3e50 0%, #1a2a3a 100%);
  color: white;
  border: 1px solid #4a5568;
}

.player-status-card.night-mode :deep(.el-card__header) {
  background-color: rgba(255, 255, 255, 0.1);
  color: white;
}

.player-status-card.night-mode :deep(.el-collapse-item__header) {
  background-color: rgba(255, 255, 255, 0.05);
  color: white;
}

.player-status-card.night-mode :deep(.el-collapse-item__content) {
  background-color: rgba(255, 255, 255, 0.05);
  color: white;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.card-header h3 {
  margin: 0;
  color: #303133;
}

.player-status-card.night-mode .card-header h3 {
  color: white;
}

.time-display {
  font-weight: bold;
  color: #e6a23c;
}

.status-content {
  display: flex;
  flex-direction: column;
  gap: 15px;
}

.status-item {
  display: flex;
  flex-direction: column;
  gap: 5px;
}

.label {
  font-weight: 500;
  font-size: 14px;
  color: #606266;
}

.player-status-card.night-mode .label {
  color: #a0aec0;
}

.value {
  font-weight: 600;
  font-size: 16px;
  color: #303133;
}

.player-status-card.night-mode .value {
  color: white;
}

.global-info {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.info-item {
  display: flex;
  justify-content: space-between;
}

:deep(.el-progress) {
  margin: 5px 0;
}

:deep(.el-collapse-item__header) {
  padding: 10px 0;
}

:deep(.el-collapse-item__content) {
  padding: 15px 0;
}
</style>