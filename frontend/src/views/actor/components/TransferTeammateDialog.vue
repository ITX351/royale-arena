<template>
  <el-dialog
    :model-value="modelValue"
    @update:model-value="$emit('update:modelValue', $event)"
    :title="`转移 ${itemName} 给队友`"
    width="420px"
  >
    <div v-if="teammates.length === 0" class="empty-state">
      暂无队友可转移
    </div>
    <el-radio-group v-else v-model="selected" class="radio-list">
      <el-radio v-for="t in teammates" :key="t.id" :value="t.id">
        {{ t.name }}
      </el-radio>
    </el-radio-group>
    <div v-if="teammates.length > 0" class="hint">对方将扣除 5 点体力</div>
    <template #footer>
      <el-button @click="$emit('update:modelValue', false)">取消</el-button>
      <el-button
        type="primary"
        :disabled="!selected || teammates.length === 0"
        @click="handleConfirm"
      >
        确认转移
      </el-button>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useGameStateStore } from '@/stores/gameState'
import type { ActorPlayer } from '@/types/gameStateTypes'

const props = defineProps<{
  modelValue: boolean
  itemName: string
  itemId: string
}>()

const emit = defineEmits<{
  'update:modelValue': [boolean]
  transferred: []
}>()

const store = useGameStateStore()
const selected = ref<string>('')

const myTeamId = computed(() => store.actorPlayer?.team_id ?? 0)
const myId = computed(() => store.actorPlayer?.id)
const teammates = computed<ActorPlayer[]>(() =>
  store.actorPlayerList.filter(
    (p) => p.id !== myId.value && p.team_id && p.team_id > 0 && p.team_id === myTeamId.value
  )
)

watch(
  () => props.modelValue,
  (open) => {
    if (open) selected.value = ''
  }
)

const handleConfirm = () => {
  if (!selected.value) return
  store.sendPlayerAction('transfer_item', {
    item_id: props.itemId,
    target_player_id: selected.value,
  })
  emit('update:modelValue', false)
  emit('transferred')
}
</script>

<style scoped>
.radio-list {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 8px;
}

.radio-list :deep(.el-radio) {
  margin-right: 0;
}
.empty-state {
  color: #909399;
  text-align: center;
  padding: 16px;
}
.hint {
  margin-top: 8px;
  color: #e6a23c;
  font-size: 12px;
}
</style>
