<template>
  <el-card class="log-message">
    <template #header>
      <div class="card-header">
        <h3>实时日志消息</h3>
      </div>
    </template>

    <div class="log-content">
      <!-- 筛选面板 -->
      <div class="filter-panel">
        <el-form :model="filterForm" layout="inline" class="filter-form">
          <el-form-item label="日期筛选">
            <el-date-picker
              v-model="filterForm.dateRange"
              type="daterange"
              range-separator="至"
              start-placeholder="开始日期"
              end-placeholder="结束日期"
              value-format="YYYY-MM-DD"
              class="date-range-picker"
            />
          </el-form-item>
          
          <el-form-item label="演员筛选">
            <el-select 
              v-model="filterForm.selectedPlayer" 
              placeholder="选择演员"
              clearable
              class="player-select"
            >
              <el-option
                v-for="player in playerOptions"
                :key="player.id"
                :label="player.name"
                :value="player.id"
              />
            </el-select>
          </el-form-item>
          
          <el-form-item label="关键词">
            <el-input
              v-model="filterForm.keyword"
              placeholder="输入关键词"
              clearable
              class="keyword-input"
            />
          </el-form-item>
          
          <el-form-item>
            <el-checkbox v-model="filterForm.showOnlyUserMessages" label="只显示用户消息" />
          </el-form-item>
          
          <el-form-item>
            <el-button type="primary" @click="applyFilter">应用筛选</el-button>
            <el-button @click="resetFilter">重置</el-button>
          </el-form-item>
        </el-form>
      </div>

      <!-- 日志消息列表 -->
      <div class="log-list" ref="logListRef">
        <div 
          v-for="message in displayedMessages" 
          :key="message.timestamp"
          :class="['log-item', message.message_type, isNewMessage(message.timestamp) ? 'fade-effect' : '']"
          :data-timestamp="message.timestamp"
        >
          <div class="log-header">
            <span class="log-timestamp">{{ formatTimestamp(message.timestamp) }}</span>
            <span class="log-type" :class="message.message_type">
              {{ getMessageTypeLabel(message.message_type) }}
            </span>
          </div>
          <div class="log-content-text">{{ message.log_message }}</div>
        </div>
        
        <!-- 空状态 -->
        <div v-if="displayedMessages.length === 0" class="empty-state">
          <el-empty description="暂无日志消息" />
        </div>
      </div>

      <!-- 显示控制 -->
      <div class="log-controls" v-if="filteredMessages.length > visibleCount">
        <el-button 
          v-if="!showAll" 
          type="primary" 
          @click="showAllMessages"
          class="show-more-btn"
        >
          显示全部 ({{ filteredMessages.length }}条)
        </el-button>
        <el-button 
          v-else 
          @click="hideExtraMessages"
          class="show-less-btn"
        >
          折叠消息
        </el-button>
      </div>
    </div>
  </el-card>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted, nextTick } from 'vue'
import { ElMessage } from 'element-plus'
import type { ActionResult } from '@/types/directorGameState'
import { formatTimestamp } from '@/utils/gameUtils'

// 定义组件属性
const props = defineProps<{
  messages: ActionResult[]
  players: Array<{ id: string; name: string }>
}>()

// 响应式状态
const filterForm = ref({
  dateRange: [] as string[],
  showOnlyUserMessages: false,
  selectedPlayer: '',
  keyword: ''
})

const visibleCount = ref(20)
const showAll = ref(false)
const logListRef = ref<HTMLElement | null>(null)
const newMessages = ref<Set<string>>(new Set())
const previousMessageTimestamps = ref<Set<string>>(new Set())

// 计算属性
const playerOptions = computed(() => {
  return props.players.map(player => ({
    id: player.id,
    name: player.name
  }))
})

const filteredMessages = computed(() => {
  let result = [...props.messages]
  
  // 日期筛选
  if (filterForm.value.dateRange.length === 2) {
    const [startDate, endDate] = filterForm.value.dateRange
    result = result.filter(message => {
      const messageDate = message.timestamp.split('T')[0]
      return messageDate >= startDate && messageDate <= endDate
    })
  }
  
  // 只显示用户消息筛选
  if (filterForm.value.showOnlyUserMessages) {
    result = result.filter(message => message.message_type === 'UserDirected')
  }
  
  // 演员筛选
  if (filterForm.value.selectedPlayer) {
    // 这里需要根据实际的消息结构来筛选，暂时假设消息中包含player_id字段
    // 由于当前设计中没有明确的player_id字段，此筛选逻辑可能需要调整
  }
  
  // 关键词筛选
  if (filterForm.value.keyword) {
    const keyword = filterForm.value.keyword.toLowerCase()
    result = result.filter(message => 
      message.log_message.toLowerCase().includes(keyword)
    )
  }
  
  // 按时间倒序排列（最新的在最上方）
  return result.sort((a, b) => 
    new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime()
  )
})

const displayedMessages = computed(() => {
  if (showAll.value) {
    return filteredMessages.value
  }
  return filteredMessages.value.slice(0, visibleCount.value)
})

// 方法实现
const getMessageTypeLabel = (type: string) => {
  const typeMap: Record<string, string> = {
    'SystemNotice': '系统消息',
    'UserDirected': '用户消息'
  }
  return typeMap[type] || type
}

const applyFilter = () => {
  ElMessage.success('筛选条件已应用')
  // 重置显示状态
  showAll.value = false
}

const resetFilter = () => {
  filterForm.value.dateRange = []
  filterForm.value.showOnlyUserMessages = false
  filterForm.value.selectedPlayer = ''
  filterForm.value.keyword = ''
  ElMessage.info('筛选条件已重置')
  // 重置显示状态
  showAll.value = false
}

const showAllMessages = () => {
  showAll.value = true
}

const hideExtraMessages = () => {
  showAll.value = false
}

// 新增方法：检查消息是否为新消息
const isNewMessage = (timestamp: string) => {
  const result = newMessages.value.has(timestamp);
  return result;
}

// 监听消息变化，标记新消息
watch(() => props.messages, (newMessagesList) => {
  // 如果没有消息，直接返回
  if (!newMessagesList || newMessagesList.length === 0) {
    return;
  }
  
  // 获取当前所有消息的时间戳
  const currentTimestamps = new Set(newMessagesList.map(msg => msg.timestamp));
  
  // 找出新增的消息（在当前消息中但不在之前的消息中的）
  const addedMessages = newMessagesList.filter(msg => !previousMessageTimestamps.value.has(msg.timestamp));
  
  // 标记新增的消息为新消息
  addedMessages.forEach(msg => {
    newMessages.value.add(msg.timestamp);
  });
  
  // 更新之前消息的时间戳集合
  previousMessageTimestamps.value = currentTimestamps;
  
  // 设置定时器在1秒后移除新消息标记（缩短一半时间）
  if (addedMessages.length > 0) {
    setTimeout(() => {
      addedMessages.forEach(msg => {
        newMessages.value.delete(msg.timestamp);
      });
    }, 1000);
  }
}, { deep: true });

// 组件挂载时的操作
onMounted(() => {
  // 初始化previousMessageTimestamps
  if (props.messages && props.messages.length > 0) {
    const initialTimestamps = new Set(props.messages.map(msg => msg.timestamp));
    previousMessageTimestamps.value = initialTimestamps;
  }
});

// 组件卸载时的操作
onUnmounted(() => {
  // 清理操作（如果需要）
});
</script>

<style scoped>
.log-message {
  margin-bottom: 20px;
  height: 100%;
  display: flex;
  flex-direction: column;
}

.card-header h3 {
  margin: 0;
  color: #303133;
}

.log-content {
  display: flex;
  flex-direction: column;
  gap: 20px;
  flex: 1;
  overflow: hidden;
}

.filter-panel {
  padding: 15px;
  background-color: #f5f7fa;
  border-radius: 4px;
}

.filter-form {
  display: flex;
  flex-wrap: wrap;
  gap: 15px;
  align-items: end;
}

.filter-form :deep(.el-form-item) {
  margin-bottom: 0;
}

.date-range-picker,
.player-select,
.keyword-input {
  width: 200px;
}

.log-list {
  flex: 1;
  max-height: none; /* 移除固定最大高度 */
  overflow-y: auto;
  border: 1px solid #e4e7ed;
  border-radius: 4px;
  padding: 10px;
}

.log-item {
  padding: 12px 15px;
  margin-bottom: 10px;
  border-radius: 4px;
  transition: all 0.3s ease;
  animation: fadeIn 0.5s ease;
  text-align: left;
}

.log-item:last-child {
  margin-bottom: 0;
}

@keyframes fadeIn {
  from {
    opacity: 0;
    transform: translateY(10px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.log-item.SystemNotice {
  background-color: #ecf5ff;
  border-left: 4px solid #409eff;
}

.log-item.UserDirected {
  background-color: #f0f9ff;
  border-left: 4px solid #67c23a;
}

/* 新增的淡入淡出效果样式 */
.log-item.fade-effect {
  background-color: #409eff !important;
  color: white !important;
  box-shadow: 0 0 15px rgba(64, 158, 255, 0.8) !important;
  transform: scale(1.02);
  animation: pulse 0.5s ease-in-out infinite alternate;
}

@keyframes pulse {
  from {
    box-shadow: 0 0 5px rgba(64, 158, 255, 0.5);
  }
  to {
    box-shadow: 0 0 20px rgba(64, 158, 255, 0.9);
  }
}

.log-header {
  display: flex;
  justify-content: space-between;
  margin-bottom: 5px;
  font-size: 12px;
  color: #909399;
}

.log-item.fade-effect .log-header {
  color: white !important;
}

.log-timestamp {
  font-weight: 500;
}

.log-type {
  padding: 2px 6px;
  border-radius: 2px;
  font-weight: 500;
}

.log-type.SystemNotice {
  background-color: #409eff;
  color: white;
}

.log-type.UserDirected {
  background-color: #67c23a;
  color: white;
}

.log-item.fade-effect .log-type {
  background-color: white !important;
  color: #409eff !important;
}

.log-content-text {
  font-size: 14px;
  line-height: 1.5;
  color: #606266;
  text-align: left;
}

.log-item.fade-effect .log-content-text {
  color: white !important;
}

.empty-state {
  text-align: center;
  padding: 40px 0;
}

.log-controls {
  display: flex;
  justify-content: center;
  padding: 15px 0;
}

.show-more-btn,
.show-less-btn {
  width: 200px;
}

@media (max-width: 768px) {
  .filter-form {
    flex-direction: column;
    align-items: stretch;
  }
  
  .date-range-picker,
  .player-select,
  .keyword-input {
    width: 100%;
  }
  
  .log-list {
    max-height: 300px;
  }
}
</style>