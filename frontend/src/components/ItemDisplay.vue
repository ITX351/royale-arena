<template>
  <div class="item-display" :class="[`rarity-${item?.rarity || 'common'}`, itemTypeClass]">
    <div class="item-icon">
      {{ getItemIcon(item?.item_type) }}
    </div>
    <div class="item-info">
      <div class="item-name">{{ item?.name || '未知物品' }}</div>
      <div class="item-properties">
        <span v-for="(value, key) in displayProperties" :key="key" class="property">
          {{ formatProperty(key, value) }}
        </span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { Item, ItemType } from '@/types/gameStateTypes'

interface Props {
  item: Item | null
}

const props = defineProps<Props>()

const itemTypeClass = computed(() => {
  if (!props.item) return 'type-unknown'
  return `type-${props.item.item_type}`
})

const displayProperties = computed(() => {
  if (!props.item?.properties) return {}
  
  // 过滤并格式化显示的属性
  const filtered: Record<string, any> = {}
  
  Object.entries(props.item.properties).forEach(([key, value]) => {
    // 只显示重要的属性
    if (['damage', 'defense', 'effect_value', 'votes', 'uses'].includes(key)) {
      filtered[key] = value
    }
  })
  
  return filtered
})

const getItemIcon = (itemType?: ItemType): string => {
  switch (itemType) {
    case 'weapon':
      return '⚔️'
    case 'equipment':
      return '🛡️'
    case 'consumable':
      return '💊'
    default:
      return '📦'
  }
}

const formatProperty = (key: string, value: any): string => {
  switch (key) {
    case 'damage':
      return `攻击:${value}`
    case 'defense':
      return `防御:${value}`
    case 'effect_value':
      return `效果:${value}`
    case 'votes':
      return `票数:${value}`
    case 'uses':
      return `耐久:${value}`
    default:
      return `${key}:${value}`
  }
}
</script>

<style scoped>
.item-display {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px;
  border-radius: 6px;
  border: 1px solid #e2e8f0;
  background: #f7fafc;
  min-width: 120px;
}

.item-icon {
  font-size: 20px;
  flex-shrink: 0;
}

.item-info {
  flex: 1;
  min-width: 0;
}

.item-name {
  font-weight: 500;
  font-size: 12px;
  color: #2d3748;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.item-properties {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
  margin-top: 2px;
}

.property {
  font-size: 10px;
  color: #718096;
  background: #edf2f7;
  padding: 1px 4px;
  border-radius: 3px;
}

/* 稀有度样式 */
.rarity-common {
  border-color: #68d391;
}

.rarity-rare {
  border-color: #4299e1;
  background: #ebf8ff;
}

.rarity-epic {
  border-color: #9f7aea;
  background: #faf5ff;
}

.rarity-legendary {
  border-color: #ed8936;
  background: #fffaf0;
}

/* 物品类型样式 */
.type-weapon .item-icon {
  color: #e53e3e;
}

.type-equipment .item-icon {
  color: #3182ce;
}

.type-consumable .item-icon {
  color: #38a169;
}

.type-unknown .item-icon {
  color: #718096;
}
</style>