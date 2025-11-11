<template>
  <el-card class="log-message">
    <template #header>
      <div class="card-header">
        <div class="header-left">
          <h3>实时日志消息</h3>
          <el-button
            type="primary"
            link
            @click="toggleFilters"
            :icon="showFilters ? ArrowUp : ArrowDown"
            class="toggle-filters-btn"
            aria-label="切换筛选面板"
          />
        </div>
        <el-button 
          type="primary" 
          size="small" 
          @click="emit('show-kill-records')"
          class="kill-records-btn"
        >
          查看击杀记录
        </el-button>
      </div>
    </template>

    <div class="log-content">
      <div class="filter-wrapper" v-show="showFilters">
        <!-- 添加禁止复制提示 -->
        <div class="copy-warning" v-if="!isDirectorView">
          <el-alert
            title="禁止复制记录到发言帖贴证"
            type="warning"
            :closable="false"
            show-icon
          />
        </div>

      <!-- 筛选面板 -->
        <div class="filter-panel">
        <el-form 
          :model="filterForm"
          layout="inline"
          class="filter-form"
          label-width="80px"
          label-position="left"
        >
          <el-form-item label="日期筛选">
            <el-date-picker
              v-model="filterForm.selectedDate"
              type="date"
              placeholder="选择日期"
              value-format="YYYY-MM-DD"
              class="date-picker"
              clearable
            />
          </el-form-item>
          
          <el-form-item label="演员筛选" v-if="isDirectorView">
            <el-select 
              v-model="filterForm.selectedPlayer" 
              placeholder="选择演员"
              filterable
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
          
          <el-form-item class="checkbox-actions">
            <div class="checkbox-actions__inner">
              <el-checkbox v-model="filterForm.showOnlyUserMessages" label="只显示用户消息" />
              <el-button @click="resetFilter">重置</el-button>
            </div>
          </el-form-item>
        </el-form>
        </div>
      </div>

      <!-- 日志消息列表 -->
      <div class="log-list" ref="logListRef">
        <div 
          v-for="message in displayedMessages" 
          :key="message.id"
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
          <!-- 更新回复按钮显示条件 -->
          <div v-if="props.isDirector && message.message_type === 'UserDirected' && isPlayerToDirectorMessage(message.log_message)" class="reply-section">
            <el-button 
              size="small" 
              type="primary" 
              @click="handleReply(message)"
              class="reply-button"
            >
              回复
            </el-button>
          </div>
        </div>
        
        <!-- 空状态 -->
        <div v-if="displayedMessages.length === 0" class="empty-state">
          <el-empty description="暂无日志消息" />
        </div>
      </div>

      <!-- 显示控制 -->
      <div class="log-controls" v-if="hasPagination">
        <el-pagination
          layout="prev, pager, next"
          :current-page="currentPage"
          :page-size="PAGE_SIZE"
          :total="filteredMessages.length"
          @current-change="handlePageChange"
        />
      </div>
    </div>
  </el-card>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'
import { ElMessage } from 'element-plus'
import type { ActionResult } from '@/types/gameStateTypes'
import { formatTimestamp } from '@/utils/gameUtils'
import { ArrowDown, ArrowUp } from '@element-plus/icons-vue'

// 定义组件属性
const props = defineProps<{
  messages: ActionResult[]
  players: Array<{ id: string; name: string }>
  isDirector?: boolean // 新增属性，用于区分是否为导演端
}>()

// 定义事件发射
const emit = defineEmits<{
  (e: 'reply-to-player', playerId: string): void,
  (e: 'show-kill-records'): void
}>()

// 响应式状态
const PAGE_SIZE = 20

const filterForm = ref({
  selectedDate: '',
  showOnlyUserMessages: false,
  selectedPlayer: '',
  keyword: ''
})

const currentPage = ref(1)
const logListRef = ref<HTMLElement | null>(null)
const newMessages = ref<Set<string>>(new Set())
const previousMessageTimestamps = ref<Set<string>>(new Set())
const showFilters = ref(false)

// 计算属性
const playerOptions = computed(() => {
  const options = props.players.map(player => ({
    id: player.id,
    name: player.name
  }))

  return options.sort((a, b) => a.name.localeCompare(b.name))
})

const isDirectorView = computed(() => props.isDirector === true)

// 工具方法：转换时间戳为本地日期字符串（YYYY-MM-DD）
const getLocalDateString = (timestamp: string) => {
  const date = new Date(timestamp)
  if (Number.isNaN(date.getTime())) {
    return ''
  }
  const offsetDate = new Date(date.getTime() - date.getTimezoneOffset() * 60000)
  return offsetDate.toISOString().split('T')[0]
}

const filteredMessages = computed(() => {
  let result = [...props.messages]
  
  // 日期筛选
  if (filterForm.value.selectedDate) {
    const selectedDate = filterForm.value.selectedDate
    result = result.filter(message => {
      const messageDate = getLocalDateString(message.timestamp)
      return messageDate === selectedDate
    })
  }
  
  // 只显示用户消息筛选
  if (filterForm.value.showOnlyUserMessages) {
    result = result.filter(message => message.message_type === 'UserDirected')
  }
  
  // 演员筛选
  if (filterForm.value.selectedPlayer) {
    const targetPlayer = props.players.find(player => player.id === filterForm.value.selectedPlayer)
    if (targetPlayer?.name) {
      result = result.filter(message => message.log_message.includes(targetPlayer.name))
    } else {
      result = []
    }
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

const totalPages = computed(() => {
  if (filteredMessages.value.length === 0) {
    return 1
  }
  return Math.ceil(filteredMessages.value.length / PAGE_SIZE)
})

const displayedMessages = computed(() => {
  if (filteredMessages.value.length === 0) {
    return []
  }
  const start = (currentPage.value - 1) * PAGE_SIZE
  return filteredMessages.value.slice(start, start + PAGE_SIZE)
})

const hasPagination = computed(() => filteredMessages.value.length > PAGE_SIZE)

// 方法实现
const getMessageTypeLabel = (type: string) => {
  const typeMap: Record<string, string> = {
    'SystemNotice': '系统消息',
    'UserDirected': '用户消息',
    'Info': '提示消息' // 这个理论上不会出现，但保留作为安全措施
  }
  return typeMap[type] || type
}

const resetFilter = () => {
  filterForm.value.selectedDate = ''
  filterForm.value.showOnlyUserMessages = false
  filterForm.value.selectedPlayer = ''
  filterForm.value.keyword = ''
  ElMessage.info('筛选条件已重置')
  // 重置分页
  currentPage.value = 1
}

const toggleFilters = () => {
  showFilters.value = !showFilters.value
}

const handlePageChange = (page: number) => {
  currentPage.value = page
}

// 新增方法：检查消息是否为新消息
const isNewMessage = (timestamp: string) => {
  const result = newMessages.value.has(timestamp);
  return result;
}

// 新增方法：提取玩家名称
const extractPlayerName = (message: string) => {
  const match = message.match(/(.*?) 向导演发送消息: /);
  return match ? match[1] : null;
}

// 新增方法：检查是否为玩家向导演发送的消息
const isPlayerToDirectorMessage = (message: string) => {
  return message.includes('向导演发送消息:');
}

// 新增方法：处理回复按钮点击
const handleReply = (message: ActionResult) => {
  const playerName = extractPlayerName(message.log_message);
  if (playerName) {
    const player = props.players.find(p => p.name === playerName);
    if (player) {
      // 触发事件，通知父组件跳转到BroadcastMessage并设置目标玩家
      emit('reply-to-player', player.id);
    }
  }
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

watch(isDirectorView, (isDirector) => {
  if (!isDirector) {
    filterForm.value.selectedPlayer = ''
  }
});

watch(filteredMessages, () => {
  if (filteredMessages.value.length === 0) {
    currentPage.value = 1
    return
  }

  const total = totalPages.value
  if (currentPage.value > total) {
    currentPage.value = total
  }
});

// 组件卸载时的操作
onUnmounted(() => {
  // 清理操作（如果需要）
});

// 自动应用筛选条件
watch(filterForm, () => {
  currentPage.value = 1
}, { deep: true });
</script>

<style scoped>
.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 8px;
}

.toggle-filters-btn {
  padding: 0;
}

.kill-records-btn {
  margin-left: auto;
}

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

.filter-wrapper {
  display: flex;
  flex-direction: column;
  gap: 12px;
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
  --filter-input-width: 200px;
}

.filter-form :deep(.el-form-item) {
  margin-bottom: 0;
}

.date-picker,
.player-select,
.keyword-input {
  width: var(--filter-input-width);
}

.filter-form :deep(.el-date-editor) {
  --el-date-editor-width: var(--filter-input-width);
}

.checkbox-actions__inner {
  display: flex;
  align-items: center;
  gap: 10px;
  width: 100%;
}

.checkbox-actions {
  margin-left: auto;
}

.checkbox-actions :deep(.el-form-item__content) {
  width: 100%;
}

.checkbox-actions__inner .el-button {
  margin-left: auto;
}

.log-list {
  flex: 1;
  max-height: none; /* 移除固定最大高度 */
  overflow-y: auto;
  border: 0px solid #e4e7ed;
  border-radius: 4px;
  padding: 0px;
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
  font-weight: bold;
}

.log-item.Info {
  background-color: #f4f4f5;
  border-left: 4px solid #909399;
}

/* 新增的淡入淡出效果样式 */
.log-item.fade-effect {
  background-color: #409eff !important;
  color: white !important;
  box-shadow: 0 0 15px rgba(64, 158, 255, 0.8) !important;
  transform: scale(1.02);
  animation: pulse 0.5s ease-in-out infinite alternate;
}

.log-item.UserDirected.fade-effect {
  font-weight: bold;
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
  user-select: none; /* Keep badge out of copy selections */
  pointer-events: none;
}

.log-type.SystemNotice {
  background-color: #409eff;
  color: white;
}

.log-type.UserDirected {
  background-color: #67c23a;
  color: white;
}

.log-type.Info {
  background-color: #909399;
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

.log-item.UserDirected .log-content-text {
  font-weight: bold;
}

.log-item.fade-effect .log-content-text {
  color: white !important;
}

.reply-section {
  margin-top: 8px;
  text-align: right;
}

.reply-button {
  padding: 4px 8px;
  font-size: 12px;
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
  
  .date-picker,
  .player-select,
  .keyword-input {
    width: 100%;
  }
  
  .log-list {
    max-height: 300px;
  }
}
</style>