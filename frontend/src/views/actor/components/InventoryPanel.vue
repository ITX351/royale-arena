<template>
  <!-- 装备槽位区域 -->
  <div class="equipment-slots">
    <div class="equipment-grid">
      <!-- 武器槽 -->
      <div class="equipment-slot weapon-slot">
        <div v-if="player?.equipped_weapon" class="slot-content">
          <el-tag type="danger" size="small">武器</el-tag>
          <div class="slot-item-name">{{ player.equipped_weapon.name }}</div>
          <div 
            v-if="hasItemProperties(player.equipped_weapon)"
            class="item-properties"
          >
            <span
              v-for="property in extractItemProperties(player.equipped_weapon)"
              :key="property.label"
            >
              {{ formatItemProperty(property) }}
            </span>
          </div>
          <el-button 
            type="warning" 
            size="small" 
            @click="unequipWeapon"
            :loading="unequippingWeapon"
          >
            卸下
          </el-button>
        </div>
        <div v-else class="slot-empty-text">
          <el-tag type="danger" size="small">武器</el-tag>
          <span>未装备武器</span>
        </div>
      </div>

      <!-- 防具槽 -->
      <div class="equipment-slot armor-slot">
        <div v-if="player?.equipped_armor" class="slot-content">
          <el-tag type="primary" size="small">防具</el-tag>
          <div class="slot-item-name">{{ player.equipped_armor.name }}</div>
          <div 
            v-if="hasItemProperties(player.equipped_armor)"
            class="item-properties"
          >
            <span
              v-for="property in extractItemProperties(player.equipped_armor)"
              :key="property.label"
            >
              {{ formatItemProperty(property) }}
            </span>
          </div>
          <el-button 
            type="warning" 
            size="small" 
            @click="unequipArmor"
            :loading="unequippingArmor"
          >
            卸下
          </el-button>
        </div>
        <div v-else class="slot-empty-text">
          <el-tag type="primary" size="small">防具</el-tag>
          <span>未装备防具</span>
        </div>
      </div>
    </div>
  </div>

  <!-- 背包物品列表 -->
  <div class="backpack-section">
    <!-- 空背包提示 -->
    <el-empty 
      v-if="!player || player.inventory.length === 0" 
      description="背包为空" 
      :image-size="80"
    />

    <!-- 背包物品列表 -->
    <div v-else class="inventory-items">
      <div 
        v-for="item in player.inventory" 
        :key="item.id"
        class="inventory-item"
      >
        <div class="item-info">
          <div class="item-name">{{ item.name }}</div>
          <div class="item-type">
            <el-tag :type="getItemTypeTagType(item.item_type?.type)" size="small">
              {{ getItemTypeLabel(item.item_type?.type) }}
            </el-tag>
          </div>
          <div v-if="hasItemProperties(item)" class="item-properties">
            <span
              v-for="property in extractItemProperties(item)"
              :key="property.label"
            >
              {{ formatItemProperty(property) }}
            </span>
          </div>
        </div>
        
        <div class="item-actions">
          <!-- 装备按钮（仅武器和防具） -->
          <el-button 
            v-if="item.item_type?.type === 'weapon' || item.item_type?.type === 'armor'" 
            type="primary" 
            size="small" 
            @click="equipItem(item.id)"
            :loading="loadingItems.includes(item.id)"
          >
            装备
          </el-button>
          
          <!-- 使用按钮（仅消耗品） -->
          <el-button 
            v-if="item.item_type?.type === 'consumable'" 
            type="success" 
            size="small" 
            @click="useItem(item.id)"
            :loading="loadingItems.includes(item.id)"
          >
            使用
          </el-button>
          
          <!-- 丢弃按钮 -->
          <el-button 
            type="danger" 
            size="small" 
            @click="discardItem(item.id)"
            :loading="loadingItems.includes(item.id)"
          >
            丢弃
          </el-button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { ElMessage } from 'element-plus'
import type { Player, Item } from '@/types/gameStateTypes'
import { getItemTypeLabel, getItemTypeTagType } from '@/utils/itemType'
import { getItemDisplayProperties, type ItemDisplayProperty, formatItemProperty } from '@/utils/itemDisplay'

const props = defineProps<{
  player: Player | null
}>()

const emit = defineEmits<{
  (e: 'equip-item', itemId: string): void
  (e: 'use-item', itemId: string): void
  (e: 'discard-item', itemId: string): void
  (e: 'unequip-weapon'): void
  (e: 'unequip-armor'): void
}>()

// 响应式数据
const loadingItems = ref<string[]>([])
const unequippingWeapon = ref(false)
const unequippingArmor = ref(false)

const extractItemProperties = (item?: Item | null): ItemDisplayProperty[] => {
  if (!item) {
    return []
  }
  return getItemDisplayProperties(item)
}

const hasItemProperties = (item?: Item | null) => {
  return extractItemProperties(item).length > 0
}

const equipItem = async (itemId: string) => {
  if (!props.player) return
  
  loadingItems.value.push(itemId)
  
  try {
    emit('equip-item', itemId)
  } catch (error) {
    ElMessage.error('装备物品失败')
  } finally {
    setTimeout(() => {
      loadingItems.value = loadingItems.value.filter(id => id !== itemId)
    }, 500)
  }
}

const useItem = async (itemId: string) => {
  if (!props.player) return
  
  loadingItems.value.push(itemId)
  
  try {
    emit('use-item', itemId)
  } catch (error) {
    ElMessage.error('使用物品失败')
  } finally {
    setTimeout(() => {
      loadingItems.value = loadingItems.value.filter(id => id !== itemId)
    }, 500)
  }
}

const discardItem = async (itemId: string) => {
  if (!props.player) return

  loadingItems.value.push(itemId)

  try {
    emit('discard-item', itemId)
  } catch (error) {
    ElMessage.error('丢弃物品失败')
  } finally {
    setTimeout(() => {
      loadingItems.value = loadingItems.value.filter(id => id !== itemId)
    }, 500)
  }
}

const unequipWeapon = async () => {
  if (!props.player?.equipped_weapon) return
  
  unequippingWeapon.value = true
  
  try {
    emit('unequip-weapon')
  } catch (error) {
    ElMessage.error('卸下武器失败')
  } finally {
    setTimeout(() => {
      unequippingWeapon.value = false
    }, 500)
  }
}

const unequipArmor = async () => {
  if (!props.player?.equipped_armor) return
  
  unequippingArmor.value = true
  
  try {
    emit('unequip-armor')
  } catch (error) {
    ElMessage.error('卸下防具失败')
  } finally {
    setTimeout(() => {
      unequippingArmor.value = false
    }, 500)
  }
}
</script>

<style scoped>
.equipment-slots {
  display: flex;
  flex-direction: column;
  gap: 16px;
  margin-bottom: 20px;
}

.equipment-slots h4 {
  margin: 0;
  color: #333;
  font-size: 14px;
  font-weight: 600;
}

.backpack-section {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.backpack-section h4 {
  margin: 0;
  color: #333;
  font-size: 14px;
  font-weight: 600;
}

.equipment-grid {
  display: flex;
  flex-direction: column;
  gap: 15px;
}

.equipment-slot {
  border: 2px solid #dee2e6;
  border-radius: 8px;
  padding: 10px;
  background-color: white;
}

.equipment-slot.weapon-slot {
  border-color: #f56c6c;
}

.equipment-slot.armor-slot {
  border-color: #409eff;
}

.slot-content {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-wrap: nowrap;
}

.slot-empty-text {
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 12px;
  color: #909399;
}

.slot-item-name {
  font-weight: 500;
  color: #333;
  max-width: 220px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.item-properties {
  display: flex;
  gap: 8px;
  font-size: 12px;
  color: #666;
}

/* 背包区域 */
.inventory-items {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.inventory-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 10px;
  border: 1px solid #dee2e6;
  border-radius: 4px;
  background-color: white;
  transition: all 0.2s;
}

.inventory-item:hover {
  box-shadow: 0 2px 6px rgba(0, 0, 0, 0.1);
}

.item-info {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-wrap: nowrap;
  min-width: 0;
}

.item-name {
  font-weight: 500;
  font-size: 14px;
  color: #333;
  max-width: 160px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.item-type {
  white-space: nowrap;
}

.item-actions {
  display: flex;
  gap: 5px;
}

@media (max-width: 768px) {
  .slot-content {
    flex-wrap: wrap;
    gap: 8px;
    align-items: flex-start;
  }

  .equipment-slots {
    gap: 12px;
    margin-bottom: 16px;
  }
  
  .slot-empty-text {
    flex-direction: column;
    align-items: flex-start;
    gap: 6px;
  }
  
  .inventory-item {
    flex-direction: column;
    align-items: flex-start;
    gap: 10px;
  }

  .item-info {
    flex-wrap: wrap;
    gap: 8px;
  }

  .item-name {
    max-width: 100%;
  }

  .item-properties {
    flex-wrap: wrap;
  }
  
  .item-actions {
    align-self: flex-end;
  }
}
</style>