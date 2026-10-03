<template>
  <el-card class="teammate-mode-card">
    <template #header>
      <div class="card-header">
        <h4>队友模式</h4>
        <el-switch v-model="masterOn" active-text="总开关" :disabled="isUpdating" />
      </div>
    </template>

    <div class="bits" :class="{ disabled: !masterOn }">
      <el-checkbox v-model="bit1" :disabled="!masterOn || isUpdating">禁止队友伤害 (位 1)</el-checkbox>
      <el-checkbox v-model="bit2" :disabled="!masterOn || isUpdating">禁止搜索到队友 (位 2)</el-checkbox>
      <el-checkbox v-model="bit4" :disabled="!masterOn || isUpdating">允许查看队友状态 (位 4)</el-checkbox>
      <el-checkbox v-model="bit8" :disabled="!masterOn || isUpdating">允许转移物品 (位 8)</el-checkbox>
    </div>

    <div class="hint" v-if="!masterOn">未开启时，所有玩家行为与散人一致。</div>
  </el-card>
</template>

<script setup lang="ts">
import { computed, onUnmounted, ref, watch } from 'vue'
import { useGameStateStore } from '@/stores/gameState'

const store = useGameStateStore()

const rulesConfig = computed(() => store.globalState?.rules_config ?? {})
const serverMode = computed(() => {
  const v = (rulesConfig.value as any).teammate_behavior
  return typeof v === 'number' ? v : 0
})
// 始终以服务器状态显示，避免发送失败后保留过期的乐观值。
const mode = serverMode
const isUpdating = ref(false)
let updateTimeout: ReturnType<typeof setTimeout> | null = null
const finishUpdate = () => {
  isUpdating.value = false
  if (updateTimeout !== null) clearTimeout(updateTimeout)
  updateTimeout = null
}
watch(serverMode, finishUpdate)
watch(() => store.connected, (connected) => { if (!connected) finishUpdate() })
onUnmounted(finishUpdate)

const setMode = (value: number) => {
  if (isUpdating.value || value === serverMode.value) return
  if (!store.connected) {
    store.setTeammateBehavior(value)
    return
  }
  isUpdating.value = true
  updateTimeout = setTimeout(finishUpdate, 5000)
  store.setTeammateBehavior(value)
}

const lastNonZero = ref(1)
watch(mode, (v) => {
  if (v !== 0) lastNonZero.value = v
}, { immediate: true })

const masterOn = computed<boolean>({
  get: () => mode.value !== 0,
  set: (v) => setMode(v ? lastNonZero.value : 0),
})

const makeBit = (bit: number) =>
  computed<boolean>({
    get: () => (mode.value & bit) !== 0,
    set: (v) => setMode(v ? mode.value | bit : mode.value & ~bit),
  })

const bit1 = makeBit(1)
const bit2 = makeBit(2)
const bit4 = makeBit(4)
const bit8 = makeBit(8)
</script>

<style scoped>
.teammate-mode-card {
  margin-bottom: 16px;
}
.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}
.card-header h4 {
  margin: 0;
}
.bits {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.bits.disabled {
  opacity: 0.5;
}
.hint {
  margin-top: 8px;
  color: #909399;
  font-size: 12px;
}
</style>
