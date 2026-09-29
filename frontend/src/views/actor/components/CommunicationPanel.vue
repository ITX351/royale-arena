<template>
  <div class="communication-panel" :class="`communication-panel--${layout}`">
    <div class="comm-row">
      <div class="deliver-group">
        <!-- 选择玩家 -->
        <div class="comm-input-line">
          <el-select
            v-model="targetPlayer"
            placeholder="选择玩家"
            size="small"
            style="width: 120px;"
            placement="bottom-start"
            :popper-options="selectPopperOptions"
            :filterable="!isCoarseSelect"
          >
            <el-option
              v-for="otherPlayer in sortedOtherPlayers"
              :key="otherPlayer.id"
              :label="otherPlayer.name"
              :value="otherPlayer.id"
            />
          </el-select>
        </div>

        <!-- 传音内容 -->
        <div class="comm-input-line">
          <el-input
            v-model="deliverMessage"
            placeholder="传音内容"
            size="small"
            style="width: 200px;"
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
        <div class="comm-input-line">
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
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import type { ActorPlayer } from '@/types/gameStateTypes'

const props = withDefaults(defineProps<{
  players: ActorPlayer[]
  selfId: string
  /** row: 桌面端单行布局；column: 移动端三行布局 */
  layout?: 'row' | 'column'
}>(), {
  layout: 'column'
})

const emit = defineEmits<{
  action: [action: string, params: Record<string, any>]
}>()

const targetPlayer = ref('')
const deliverMessage = ref('')
const directorMessage = ref('')
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

// 移动端/触屏设备禁用下拉框筛选输入，避免点击时弹出输入法或出现可编辑的搜索框
const MOBILE_SELECT_BREAKPOINT = 768
const isCoarsePointer = ref(false)
const isNarrowViewport = ref(false)
const isCoarseSelect = computed(() => isCoarsePointer.value || isNarrowViewport.value)

let coarsePointerMediaQuery: MediaQueryList | null = null
const handleCoarsePointerChange = (event: MediaQueryListEvent) => {
  isCoarsePointer.value = event.matches
}

const updateViewportWidth = () => {
  if (typeof window !== 'undefined') {
    isNarrowViewport.value = window.innerWidth <= MOBILE_SELECT_BREAKPOINT
  }
}

const otherPlayers = computed((): ActorPlayer[] => {
  return props.players.filter(p => p.id !== props.selfId)
})

const sortedOtherPlayers = computed(() => {
  return [...otherPlayers.value].sort((a, b) => {
    const localeResult = a.name.localeCompare(b.name, 'zh-CN-u-co-pinyin')
    return localeResult || a.name.localeCompare(b.name)
  })
})

const deliverMessageTooLong = computed(() => {
  return deliverMessage.value.length > MESSAGE_MAX_LENGTH
})

const directorMessageTooLong = computed(() => {
  return directorMessage.value.length > MESSAGE_MAX_LENGTH
})

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

onMounted(() => {
  if (typeof window !== 'undefined') {
    updateViewportWidth()
    window.addEventListener('resize', updateViewportWidth)

    coarsePointerMediaQuery = window.matchMedia('(pointer: coarse)')
    isCoarsePointer.value = coarsePointerMediaQuery.matches
    coarsePointerMediaQuery.addEventListener('change', handleCoarsePointerChange)
  }
})

onUnmounted(() => {
  if (typeof window !== 'undefined') {
    window.removeEventListener('resize', updateViewportWidth)
  }
  coarsePointerMediaQuery?.removeEventListener('change', handleCoarsePointerChange)
  coarsePointerMediaQuery = null
})
</script>

<style scoped>
.communication-panel {
  width: 100%;
  background: #ffffff;
  border-radius: 8px;
  padding: 16px;
  border: 1px solid #e1e6f0;
}

.comm-row {
  display: flex;
  gap: 16px;
  align-items: flex-start;
  flex-wrap: wrap;
  width: 100%;
}

.deliver-group {
  display: flex;
  flex-direction: column;
  gap: 10px;
  align-items: flex-start;
  width: 100%;
}

.comm-input-line {
  display: flex;
  gap: 8px;
  align-items: center;
}

.deliver-group .el-input {
  flex: none;
}

.input-error {
  color: #f56c6c;
  font-size: 12px;
  white-space: nowrap;
}

/* 桌面版：三个组件放在同一行 */
.communication-panel--row .deliver-group {
  flex-direction: row;
  align-items: center;
  flex-wrap: wrap;
  gap: 20px;
}

/* 移动端：保持三行布局 */
@media (max-width: 600px) {
  .communication-panel--column .deliver-group {
    width: 100%;
  }

  .communication-panel--column .comm-input-line {
    flex-direction: row;
    flex-wrap: wrap;
    width: 100%;
    justify-content: left;
  }

  .communication-panel--column .deliver-group .el-select {
    flex: 1 1 100%;
    width: auto !important;
  }

  .communication-panel--column .comm-input-line .el-input {
    flex: 1 1 140px;
    width: auto !important;
  }

  .communication-panel--column .comm-input-line .el-button {
    flex: 0 0 auto;
  }
}

:deep(.el-input__inner),
:deep(.el-select .el-input__inner),
:deep(.el-select__selected-item) {
  font-size: 12px;
}
</style>
