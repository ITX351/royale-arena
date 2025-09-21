<template>
  <el-card class="inventory-panel">
    <template #header>
      <div class="card-header">
        <h3>背包</h3>
        <el-tag v-if="player" type="info">物品数量: {{ player.inventory.length }}</el-tag>
      </div>
    </template>

    <div class="inventory-content">
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
          :class="{ 'equipped': isEquipped(item.id), 'hand-item': isHandItem(item.id) }"
        >
          <div class="item-info">
            <div class="item-name">{{ item.name }}</div>
            <div class="item-type">
              <el-tag :type="getItemTypeTagType(item.item_type)">
                {{ getItemTypeLabel(item.item_type) }}
              </el-tag>
            </div>
          </div>
          
          <div class="item-actions">
            <!-- 装备按钮 -->
            <el-button 
              v-if="canEquipItem(item)" 
              type="primary" 
              size="small" 
              @click="equipItem(item.id)"
              :loading="loadingItems.includes(item.id)"
            >
              装备
            </el-button>
            
            <!-- 使用按钮 -->
            <el-button 
              v-else-if="canUseItem(item)" 
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

      <!-- 手持物品显示 -->
      <div v-if="player && player.hand_item" class="hand-item-section">
        <h4>手持物品</h4>
        <div class="hand-item-display">
          <el-tag type="success" size="large">
            {{ getHandItemName() }}
          </el-tag>
          <el-button 
            type="warning" 
            size="small" 
            @click="putDownHandItem"
            :loading="puttingDownHandItem"
          >
            放下
          </el-button>
        </div>
      </div>
    </div>
  </el-card>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import type { Player, Item } from '@/types/gameStateTypes'

const props = defineProps<{
  player: Player | null
}>()

const emit = defineEmits<{
  (e: 'equip-item', itemId: string): void
  (e: 'use-item', itemId: string): void
  (e: 'discard-item', itemId: string): void
  (e: 'put-down-hand-item'): void
}>()

// 响应式数据
const loadingItems = ref<string[]>([])
const puttingDownHandItem = ref(false)

// 计算属性
const isEquipped = computed(() => (itemId: string) => {
  return props.player?.equipped_item === itemId
})

const isHandItem = computed(() => (itemId: string) => {
  return props.player?.hand_item === itemId
})

// 方法
const getItemTypeLabel = (itemType: string) => {
  switch (itemType) {
    case 'weapon': return '武器'
    case 'consumable': return '消耗品'
    case 'equipment': return '装备'
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

const canEquipItem = (item: Item) => {
  // 只有装备和武器可以装备
  return (item.item_type === 'equipment' || item.item_type === 'weapon') && 
         props.player?.equipped_item !== item.id
}

const canUseItem = (item: Item) => {
  // 只有消耗品可以使用
  return item.item_type === 'consumable'
}

const getHandItemName = () => {
  if (!props.player || !props.player.hand_item) return ''
  
  const handItem = props.player.inventory.find(item => item.id === props.player?.hand_item)
  return handItem ? handItem.name : '未知物品'
}

const equipItem = async (itemId: string) => {
  if (!props.player) return
  
  // 设置加载状态
  loadingItems.value.push(itemId)
  
  try {
    // 发送装备事件
    emit('equip-item', itemId)
  } catch (error) {
    ElMessage.error('装备物品失败')
  } finally {
    // 移除加载状态
    loadingItems.value = loadingItems.value.filter(id => id !== itemId)
  }
}

const useItem = async (itemId: string) => {
  if (!props.player) return
  
  // 设置加载状态
  loadingItems.value.push(itemId)
  
  try {
    // 发送使用事件
    emit('use-item', itemId)
  } catch (error) {
    ElMessage.error('使用物品失败')
  } finally {
    // 移除加载状态
    loadingItems.value = loadingItems.value.filter(id => id !== itemId)
  }
}

const discardItem = async (itemId: string) => {
  if (!props.player) return
  
  // 确认丢弃
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
    
    // 设置加载状态
    loadingItems.value.push(itemId)
    
    try {
      // 发送丢弃事件
      emit('discard-item', itemId)
    } catch (error) {
      ElMessage.error('丢弃物品失败')
    } finally {
      // 移除加载状态
      loadingItems.value = loadingItems.value.filter(id => id !== itemId)
    }
  } catch {
    // 用户取消操作
    return
  }
}

const putDownHandItem = async () => {
  if (!props.player || !props.player.hand_item) return
  
  puttingDownHandItem.value = true
  
  try {
    // 发送放下手持物品事件
    emit('put-down-hand-item')
  } catch (error) {
    ElMessage.error('放下物品失败')
  } finally {
    puttingDownHandItem.value = false
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
  gap: 15px;
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

.inventory-item.equipped {
  border-color: #409eff;
  background-color: #ecf5ff;
}

.inventory-item.hand-item {
  border-color: #67c23a;
  background-color: #f0f9eb;
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

.hand-item-section h4 {
  margin: 0 0 10px 0;
  color: #333;
  font-size: 14px;
}

.hand-item-display {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 10px;
  border: 1px solid #d1edc4;
  border-radius: 4px;
  background-color: #f0f9eb;
}

@media (max-width: 768px) {
  .inventory-item {
    flex-direction: column;
    align-items: flex-start;
    gap: 10px;
  }
  
  .item-actions {
    align-self: flex-end;
  }
  
  .hand-item-display {
    flex-direction: column;
    align-items: flex-start;
    gap: 10px;
  }
}
</style>