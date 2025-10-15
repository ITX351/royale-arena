<template>
  <el-card class="broadcast-message">
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
            />
          </el-form-item>

          <el-form-item label="发送目标" prop="targetType">
            <el-radio-group v-model="broadcastForm.targetType">
              <el-radio label="all">广播到所有玩家</el-radio>
              <el-radio label="player">发送给特定玩家</el-radio>
            </el-radio-group>
          </el-form-item>

          <el-form-item 
            v-if="broadcastForm.targetType === 'player'" 
            label="选择玩家" 
            prop="targetPlayer"
          >
            <el-select 
              v-model="broadcastForm.targetPlayer" 
              placeholder="请选择玩家"
              style="width: 100%"
              filterable
            >
              <el-option
                v-for="player in players"
                :key="player.id"
                :label="player.name"
                :value="player.id"
              />
            </el-select>
          </el-form-item>

          <el-form-item>
            <el-button 
              type="primary" 
              @click="sendBroadcast"
              :loading="sending"
            >
              发送消息
            </el-button>
          </el-form-item>
        </el-form>
      </div>
    </el-collapse-transition>
  </el-card>
</template>

<script setup lang="ts">
import { ref, reactive } from 'vue'
import { ElMessage, ElForm } from 'element-plus'
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
const messageInputRef = ref<HTMLInputElement | null>(null)

const sending = ref(false)

// 新增方法：设置目标玩家
function setTargetPlayer(playerId: string) {
  broadcastForm.targetType = 'player'
  broadcastForm.targetPlayer = playerId
}

// 新增方法：聚焦到消息输入框
function focusMessageInput() {
  // 等待DOM更新后聚焦
  setTimeout(() => {
    if (messageInputRef.value) {
      messageInputRef.value.focus();
    }
  }, 100);
}

// 新增方法：展开面板
function expandPanel() {
  isCollapsed.value = false;
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
</style>