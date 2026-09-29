<template>
  <el-card class="teammate-mode-card">
    <template #header>
      <div class="card-header">
        <h4>队友模式</h4>
        <el-switch v-model="masterOn" active-text="总开关" />
      </div>
    </template>

    <div class="bits" :class="{ disabled: !masterOn }">
      <el-checkbox v-model="bit1" :disabled="!masterOn">禁止队友伤害 (位 1)</el-checkbox>
      <el-checkbox v-model="bit2" :disabled="!masterOn">禁止搜索到队友 (位 2)</el-checkbox>
      <el-checkbox v-model="bit8" :disabled="!masterOn">允许转移物品 (位 8)</el-checkbox>
    </div>

    <div class="hint" v-if="!masterOn">未开启时，所有玩家行为与散人一致。</div>
  </el-card>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useGameStateStore } from '@/stores/gameState'

const store = useGameStateStore()

const rulesConfig = computed(() => store.globalState?.rules_config ?? {})
const mode = computed(() => {
  const v = (rulesConfig.value as any).teammate_behavior
  return typeof v === 'number' ? v : 0
})

const lastNonZero = ref(1)
watch(mode, (v) => {
  if (v !== 0) lastNonZero.value = v
}, { immediate: true })

const masterOn = computed<boolean>({
  get: () => mode.value !== 0,
  set: (v) => store.setTeammateBehavior(v ? lastNonZero.value : 0),
})

const makeBit = (bit: number) =>
  computed<boolean>({
    get: () => (mode.value & bit) !== 0,
    set: (v) => store.setTeammateBehavior(v ? mode.value | bit : mode.value & ~bit),
  })

const bit1 = makeBit(1)
const bit2 = makeBit(2)
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
