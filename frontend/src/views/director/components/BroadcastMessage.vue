<template>
  <el-card
    class="broadcast-message collapsible-card"
    :class="{ 'collapsible-card--collapsed': isCollapsed }"
  >
    <template #header>
      <div class="card-header">
        <h3>广播消息</h3>
        <el-button 
          type="primary" 
          size="small" 
          @click="isCollapsed = !isCollapsed"
          :icon="isCollapsed ? ArrowDown : ArrowUp"
          circle
        />
      </div>
    </template>

    <el-collapse-transition>
      <div v-show="!isCollapsed" class="broadcast-content">
        <el-form :model="broadcastForm" ref="broadcastFormRef" label-width="80px">
          <el-form-item label="消息内容" prop="message">
            <el-input
              v-model="broadcastForm.message"
              type="textarea"
              :rows="4"
              placeholder="请输入要广播的消息内容"
              maxlength="500"
              show-word-limit
              ref="messageInputRef"
              @keydown.enter.ctrl.prevent="handleCtrlEnter"
            />
          </el-form-item>

          <el-form-item label="发送目标" prop="targetType">
            <el-radio-group v-model="broadcastForm.targetType">
              <el-radio label="all">广播到所有玩家</el-radio>
              <el-radio label="player" class="target-player-radio">
                发送给特定玩家
                <el-select 
                  v-if="broadcastForm.targetType === 'player'"
                  v-model="broadcastForm.targetPlayer" 
                  placeholder="请选择玩家"
                  class="target-player-select"
                  filterable
                >
                  <el-option
                    v-for="player in sortedPlayers"
                    :key="player.id"
                    :label="player.name"
                    :value="player.id"
                  />
                </el-select>
              </el-radio>
            </el-radio-group>
          </el-form-item>

          <el-form-item>
            <div class="send-row">
              <el-button 
                type="primary" 
                @click="sendBroadcast"
                :loading="sending"
              >
                发送消息
              </el-button>
              <span class="shortcut-hint">Ctrl + Enter 快速发送</span>
            </div>
          </el-form-item>
        </el-form>
      </div>
    </el-collapse-transition>
  </el-card>
</template>

<script setup lang="ts">
import { ref, reactive, computed, nextTick } from 'vue'
import { ElMessage, ElForm } from 'element-plus'
import type { InputInstance } from 'element-plus'
import { ArrowUp, ArrowDown } from '@element-plus/icons-vue'
import { useGameStateStore } from '@/stores/gameState'
import type { Player } from '@/types/gameStateTypes'

// 定义组件属性
const props = defineProps<{
  gameId: string
  players: Player[]
}>()

// 定义事件发射
const emit = defineEmits<{
  (e: 'message-sent', message: string, targetType: 'all' | 'player', targetPlayer?: string): void
}>()

// 定义暴露给父组件的方法
defineExpose({
  setTargetPlayer,
  focusMessageInput,
  expandPanel // 新增方法，用于展开面板
})

// 获取状态管理器
const store = useGameStateStore()

// 折叠状态，默认展开
const isCollapsed = ref(false)

// 响应式状态
const broadcastFormRef = ref<InstanceType<typeof ElForm>>()
const broadcastForm = reactive({
  message: '',
  targetType: 'all' as 'all' | 'player',
  targetPlayer: ''
})

// 添加对消息输入框的引用
const messageInputRef = ref<InputInstance>()

const sending = ref(false)

const sortedPlayers = computed(() => {
  return [...props.players].sort((a, b) => {
    const localeResult = a.name.localeCompare(b.name, 'zh-CN-u-co-pinyin')
    return localeResult || a.name.localeCompare(b.name)
  })
})

// 新增方法：设置目标玩家
function setTargetPlayer(playerId: string) {
  broadcastForm.targetType = 'player'
  broadcastForm.targetPlayer = playerId
}

// 新增方法：聚焦到消息输入框
function focusMessageInput() {
  // 使用 nextTick 确保在 DOM 更新后再聚焦
  nextTick(() => {
    messageInputRef.value?.focus()
  })
}

// 新增方法：展开面板
function expandPanel() {
  isCollapsed.value = false
}

// 新增方法：处理 Ctrl + Enter 快捷发送
function handleCtrlEnter() {
  if (sending.value) return
  sendBroadcast()
}

// 方法实现
const sendBroadcast = async () => {
  if (!broadcastFormRef.value) return
  
  sending.value = true
  
  // 手动检查是否填写了消息内容
  if (!broadcastForm.message.trim()) {
    ElMessage.warning('请输入消息内容')
    sending.value = false
    return
  }
  
  if (broadcastForm.targetType === 'all') {
    // 广播给所有玩家
    store.sendBroadcast(broadcastForm.message)
    ElMessage.success('消息已广播给所有玩家')
    emit('message-sent', broadcastForm.message, 'all')
  } else {
    // 发送给特定玩家
    if (broadcastForm.targetPlayer) {
      store.sendDirectorMessageToPlayer(broadcastForm.targetPlayer, broadcastForm.message)
      const targetPlayer = props.players.find(p => p.id === broadcastForm.targetPlayer)
      ElMessage.success(`消息已发送给玩家: ${targetPlayer?.name}`)
      emit('message-sent', broadcastForm.message, 'player', broadcastForm.targetPlayer)
    } else {
      ElMessage.warning('请选择要发送消息的玩家')
      sending.value = false
      return
    }
  }
  
  // 重置表单
  broadcastForm.message = ''
  broadcastForm.targetType = 'all'
  broadcastForm.targetPlayer = ''
  
  sending.value = false
}
</script>

<style scoped>
.broadcast-message {
  margin-bottom: 20px;
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

.broadcast-content {
  padding: 20px 0;
}

.target-player-radio {
  display: inline-flex;
  align-items: center;
}

.target-player-select {
  width: 140px;
  margin-left: 20px;
}

.shortcut-hint {
  margin-left: 12px;
  color: #909399;
  font-size: 12px;
  white-space: nowrap;
}

.send-row {
  display: inline-flex;
  align-items: center;
}
</style>