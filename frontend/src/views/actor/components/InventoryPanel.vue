<template>
  <el-card class="inventory-panel">
    <template #header>
      <div class="card-header">
        <h3>物品管理</h3>
        <el-tag v-if="player" type="info">总物品数: {{ getTotalItemCount() }}</el-tag>
      </div>
    </template>

    <div class="inventory-content">
      <!-- 装备槽位区域 -->
      <div class="equipment-slots">
        <h4>装备槽位</h4>
        
        <div class="equipment-grid">
          <!-- 武器槽 -->
          <div class="equipment-slot weapon-slot">
            <div class="slot-header">
              <el-tag type="danger" size="small">武器</el-tag>
            </div>
            <div v-if="player?.equipped_weapon" class="slot-content">
              <div class="item-info">
                <div class="item-name">{{ player.equipped_weapon.name }}</div>
                <div class="item-properties">
                  <span v-if="player.equipped_weapon.properties.damage">
                    伤害: {{ player.equipped_weapon.properties.damage }}
                  </span>
                </div>
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
            <div v-else class="slot-empty">
              <el-empty description="未装备武器" :image-size="50" />
            </div>
          </div>

          <!-- 防具槽 -->
          <div class="equipment-slot armor-slot">
            <div class="slot-header">
              <el-tag type="primary" size="small">防具</el-tag>
            </div>
            <div v-if="player?.equipped_armor" class="slot-content">
              <div class="item-info">
                <div class="item-name">{{ player.equipped_armor.name }}</div>
                <div class="item-properties">
                  <span v-if="player.equipped_armor.properties.defense">
                    防御: {{ player.equipped_armor.properties.defense }}
                  </span>
                </div>
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
            <div v-else class="slot-empty">
              <el-empty description="未装备防具" :image-size="50" />
            </div>
          </div>
        </div>
      </div>

      <!-- 背包物品列表 -->
      <div class="backpack-section">
        <h4>背包</h4>
        
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
                <el-tag :type="getItemTypeTagType(item.item_type)" size="small">
                  {{ getItemTypeLabel(item.item_type) }}
                </el-tag>
              </div>
              <div v-if="hasItemProperties(item)" class="item-properties">
                <span v-if="item.properties.damage">伤害: {{ item.properties.damage }}</span>
                <span v-if="item.properties.defense">防御: {{ item.properties.defense }}</span>
                <span v-if="item.properties.effect_value">效果: {{ item.properties.effect_value }}</span>
              </div>
            </div>
            
            <div class="item-actions">
              <!-- 装备按钮（仅武器和防具） -->
              <el-button 
                v-if="item.item_type === 'weapon' || item.item_type === 'equipment'" 
                type="primary" 
                size="small" 
                @click="equipItem(item.id)"
                :loading="loadingItems.includes(item.id)"
              >
                装备
              </el-button>
              
              <!-- 使用按钮（仅消耗品） -->
              <el-button 
                v-if="item.item_type === 'consumable'" 
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
    </div>
  </el-card>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import type { Player, Item } from '@/types/gameStateTypes'

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

// 方法
const getTotalItemCount = () => {
  if (!props.player) return 0
  let count = props.player.inventory.length
  if (props.player.equipped_weapon) count++
  if (props.player.equipped_armor) count++
  return count
}

const getItemTypeLabel = (itemType: string) => {
  switch (itemType) {
    case 'weapon': return '武器'
    case 'consumable': return '消耗品'
    case 'equipment': return '防具'
    default: return '未知'
  }
}

const getItemTypeTagType = (itemType: string) => {
  switch (itemType) {
    case 'weapon': return 'danger'
    case 'consumable': return 'success'
    case 'equipment': return 'primary'
    default: return 'info'
  }
}

const hasItemProperties = (item: Item) => {
  return item.properties && Object.keys(item.properties).length > 0
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
  
  try {
    await ElMessageBox.confirm(
      '确定要丢弃这个物品吗？此操作不可撤销。',
      '确认丢弃',
      {
        confirmButtonText: '确定',
        cancelButtonText: '取消',
        type: 'warning',
      }
    )
    
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
  } catch {
    return
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
.inventory-panel {
  background-color: #f8f9fa;
  border: 1px solid #e9ecef;
}

.inventory-panel :deep(.el-card__header) {
  background-color: #e9ecef;
  padding: 10px 15px;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.card-header h3 {
  margin: 0;
  color: #333;
  font-size: 16px;
}

.inventory-content {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

/* 装备槽位区域 */
.equipment-slots h4 {
  margin: 0 0 10px 0;
  color: #333;
  font-size: 14px;
  font-weight: 600;
}

.equipment-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
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

.slot-header {
  margin-bottom: 10px;
}

.slot-content {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.slot-empty {
  min-height: 100px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.item-properties {
  font-size: 12px;
  color: #666;
  margin-top: 4px;
}

.item-properties span {
  margin-right: 8px;
}

/* 背包区域 */
.backpack-section h4 {
  margin: 0 0 10px 0;
  color: #333;
  font-size: 14px;
  font-weight: 600;
}

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
  flex-direction: column;
  gap: 5px;
}

.item-name {
  font-weight: 500;
  font-size: 14px;
  color: #333;
}

.item-type {
  margin-top: 2px;
}

.item-actions {
  display: flex;
  gap: 5px;
}

@media (max-width: 768px) {
  .equipment-grid {
    grid-template-columns: 1fr;
  }
  
  .inventory-item {
    flex-direction: column;
    align-items: flex-start;
    gap: 10px;
  }
  
  .item-actions {
    align-self: flex-end;
  }
}
</style>