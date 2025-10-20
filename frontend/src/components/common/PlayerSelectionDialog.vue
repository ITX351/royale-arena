<template>
  <el-dialog
    v-model="dialogVisible"
    :title="title"
    :width="width"
    @close="handleClose"
  >
    <el-form label-width="80px">
      <el-form-item :label="playerLabel">
        <el-select
          v-model="selectedPlayers"
          multiple
          filterable
          collapse-tags
          :placeholder="placeholder"
          :style="{ width: '100%' }"
          :multiple-limit="maxSelection > 0 ? maxSelection : 0"
        >
          <el-option
            v-for="player in playerOptions"
            :key="player.id"
            :label="player.name"
            :value="player.id"
          />
        </el-select>
      </el-form-item>
      <p class="selection-hint">
        已选择 {{ selectedPlayers.length }} / {{ displayLimit }}
      </p>
    </el-form>
    <template #footer>
      <span class="dialog-footer">
        <el-button @click="handleCancel">取消</el-button>
        <el-button
          type="primary"
          @click="handleConfirm"
          :disabled="confirmDisabled"
        >
          确定
        </el-button>
      </span>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import type { ActorPlayer } from '@/types/gameStateTypes'

const props = withDefaults(defineProps<{
  modelValue: boolean
  players: ActorPlayer[]
  maxSelection?: number
  minSelection?: number
  title?: string
  playerLabel?: string
  placeholder?: string
  width?: string
  initialSelected?: string[]
}>(), {
  players: () => [],
  maxSelection: 1,
  minSelection: 1,
  title: '选择玩家',
  playerLabel: '玩家',
  placeholder: '请选择玩家',
  width: '420px',
  initialSelected: () => []
})

const emit = defineEmits<{
  'update:modelValue': [value: boolean]
  confirm: [selectedPlayerIds: string[]]
  cancel: []
}>()

const selectedPlayers = ref<string[]>([...new Set(props.initialSelected)])

const dialogVisible = computed({
  get: () => props.modelValue,
  set: value => emit('update:modelValue', value)
})

const playerOptions = computed(() => {
  return [...props.players].sort((a, b) => a.name.localeCompare(b.name, 'zh-CN'))
})

const displayLimit = computed(() => {
  if (props.maxSelection && props.maxSelection > 0) {
    return props.maxSelection
  }
  return '不限'
})

const confirmDisabled = computed(() => {
  if (props.minSelection && selectedPlayers.value.length < props.minSelection) {
    return true
  }
  if (props.maxSelection && props.maxSelection > 0) {
    return selectedPlayers.value.length === 0 || selectedPlayers.value.length > props.maxSelection
  }
  return selectedPlayers.value.length === 0
})

watch(
  () => props.modelValue,
  newVal => {
    if (newVal) {
      selectedPlayers.value = [...new Set(props.initialSelected)]
    }
  }
)

const handleConfirm = () => {
  if (confirmDisabled.value) return
  emit('confirm', [...selectedPlayers.value])
  dialogVisible.value = false
}

const handleCancel = () => {
  emit('cancel')
  dialogVisible.value = false
}

const handleClose = () => {
  handleCancel()
}
</script>

<style scoped>
.selection-hint {
  margin: 4px 0 0 0;
  font-size: 12px;
  color: #909399;
}

.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
}
</style>
