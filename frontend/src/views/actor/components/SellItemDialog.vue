<template>
  <el-dialog
    :model-value="modelValue"
    @update:model-value="$emit('update:modelValue', $event)"
    title="售出道具"
    width="460px"
  >
    <div v-if="sellableItems.length === 0" class="empty-state">
      没有可售出的道具（只有已配置售出价格的武器和防具可售）
    </div>
    <el-checkbox-group v-else v-model="selected" class="check-list">
      <el-checkbox v-for="it in sellableItems" :key="it.id" :value="it.id">
        {{ it.name }}
        <span class="price">→ {{ formatPrice(it.price) }} 币</span>
      </el-checkbox>
    </el-checkbox-group>
    <div v-if="validationError" class="pair-hint">{{ validationError }}</div>
    <template #footer>
      <el-button @click="$emit('update:modelValue', false)">取消</el-button>
      <el-button type="primary" :disabled="!canConfirm" @click="handleConfirm">
        确认售出
      </el-button>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useGameStateStore } from '@/stores/gameState'
import type { Item } from '@/types/gameStateTypes'

const props = defineProps<{
  modelValue: boolean
  inventory: Item[]
}>()

const emit = defineEmits<{
  'update:modelValue': [boolean]
}>()

const store = useGameStateStore()
const selected = ref<string[]>([])

interface SellableItem {
  id: string
  name: string
  price: number
  rarity: string | null
}

const sellableItems = computed<SellableItem[]>(() => {
  const priceOf = (rarity: string | null) =>
    rarity ? store.sellPrices.find((e) => e.rarity === rarity)?.price : undefined
  return props.inventory
    .filter(
      (it) =>
        (it.item_type?.type === 'weapon' || it.item_type?.type === 'armor') &&
        priceOf(it.rarity) !== undefined
    )
    .map((it) => ({ id: it.id, name: it.name, price: priceOf(it.rarity) as number, rarity: it.rarity }))
})

const selectedItems = computed(() =>
  sellableItems.value.filter((it) => selected.value.includes(it.id))
)

const validationError = computed(() => {
  const greens = selectedItems.value.filter((it) => it.rarity === 'common')
  const others = selectedItems.value.filter((it) => it.rarity !== 'common')
  if (greens.length === 1 && others.length === 0)
    return '绿色物品需成对售出，请再勾选 1 件绿色物品'
  if (greens.length >= 1 && others.length >= 1) return '绿色物品不能与其他稀有度混合售出'
  if (others.length >= 2) return '非绿色物品一次只能售出 1 件'
  if (greens.length > 2) return '绿色物品一次只能售出 2 件'
  return ''
})

const canConfirm = computed(
  () => selectedItems.value.length > 0 && validationError.value === ''
)

const formatPrice = (p: number) => String(p)

watch(
  () => props.modelValue,
  (open) => {
    if (open) selected.value = []
  }
)

const handleConfirm = () => {
  if (!canConfirm.value) return
  store.sellItem(selected.value)
  emit('update:modelValue', false)
}
</script>

<style scoped>
.check-list {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 8px;
}
.check-list :deep(.el-checkbox) {
  margin-right: 0;
}
.pair-hint {
  margin-top: 8px;
  color: #e6a23c;
  font-size: 12px;
}
.price {
  margin-left: 8px;
  color: #e6a23c;
  font-size: 12px;
}
.empty-state {
  color: #909399;
  text-align: center;
  padding: 16px;
}
</style>
