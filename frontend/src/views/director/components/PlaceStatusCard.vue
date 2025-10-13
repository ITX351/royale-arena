<template>
  <el-card class="place-status-card">
    <template #header>
      <div class="card-header">
        <h3>地点状态管理</h3>
        <el-button 
          type="danger" 
          size="small" 
          @click="handleClearAllItems"
          :disabled="!hasAnyItems"
        >
          清空全场物品
        </el-button>
      </div>
    </template>
    <div class="place-status-content">
      <el-table :data="placeList" style="width: 100%" size="small" max-height="500">
        <el-table-column prop="name" label="地点名称" width="120" />
        <el-table-column label="状态" width="120">
          <template #default="scope">
            <el-switch
              v-model="scope.row.is_destroyed"
              active-text="已摧毁"
              inactive-text="未摧毁"
              @change="val => handlePlaceStatusChange(scope.row.name, val)"
              size="small"
            />
          </template>
        </el-table-column>
        <el-table-column label="玩家列表" min-width="150">
          <template #default="scope">
            <div class="players-list">
              <el-tag 
                v-for="playerId in scope.row.players" 
                :key="playerId" 
                size="small"
                class="player-tag"
              >
                {{ getPlayerName(playerId) }}
              </el-tag>
              <span v-if="scope.row.players.length === 0" class="empty-text">无</span>
            </div>
          </template>
        </el-table-column>
        <el-table-column label="物品列表" min-width="250">
          <template #default="scope">
            <div class="items-list">
              <div 
                v-for="(item, index) in scope.row.items" 
                :key="index" 
                class="item-row"
              >
                <span class="item-name">{{ item.name }}</span>
                <el-button 
                  type="danger" 
                  size="small" 
                  @click="handleDeleteItem(scope.row.name, item.name)"
                  :icon="Delete"
                  circle
                />
              </div>
              <div v-if="scope.row.items.length === 0" class="empty-text">无</div>
              <el-button 
                v-if="scope.row.items.length > 0"
                type="warning" 
                size="small" 
                @click="handleClearPlaceItems(scope.row.name)"
                class="clear-place-btn"
              >
                清空地点
              </el-button>
            </div>
          </template>
        </el-table-column>
      </el-table>
    </div>
  </el-card>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { ElMessageBox, ElMessage } from 'element-plus'
import { Delete } from '@element-plus/icons-vue'
import { useGameStateStore } from '@/stores/gameState'
import type { DirectorPlace as Place } from '@/types/gameStateTypes'

// 定义组件属性
const props = defineProps<{
  places: Place[]
}>()

// 定义事件发射
const emit = defineEmits<{
  (e: 'place-status-change', placeName: string, isDestroyed: boolean): void
}>()

const store = useGameStateStore()

// 计算属性
const placeList = computed<Place[]>(() => {
  return props.places
})

// 检查是否有任何物品
const hasAnyItems = computed(() => {
  return placeList.value.some(place => place.items.length > 0)
})

// 获取玩家名称
const getPlayerName = (playerId: string): string => {
  const player = store.directorPlayers[playerId]
  return player ? player.name : playerId
}

// 地点状态调整方法
const handlePlaceStatusChange = (placeName: string, isDestroyed: boolean | string | number) => {
  // 确俛isDestroyed是布尔值
  const isDestroyedBool = Boolean(isDestroyed)
  // 调用store中的方法调整地点状态
  store.togglePlaceStatus(placeName, isDestroyedBool)
  // 发送事件通知父组件
  emit('place-status-change', placeName, isDestroyedBool)
}

// 删除单个物品
const handleDeleteItem = async (placeName: string, itemName: string) => {
  try {
    await ElMessageBox.confirm(
      `确定要删除地点「${placeName}」中的物品「${itemName}」吗？`,
      '确认删除',
      {
        confirmButtonText: '确定',
        cancelButtonText: '取消',
        type: 'warning'
      }
    )
    
    // 发送删除请求
    store.sendBatchItemDeletion([{ place_name: placeName, item_name: itemName }], false)
    ElMessage.success('删除请求已发送')
  } catch {
    // 用户取消操作
  }
}

// 清空地点所有物品
const handleClearPlaceItems = async (placeName: string) => {
  try {
    await ElMessageBox.confirm(
      `确定要清空地点「${placeName}」中的所有物品吗？`,
      '确认清空',
      {
        confirmButtonText: '确定',
        cancelButtonText: '取消',
        type: 'warning'
      }
    )
    
    // 发送清空请求（item_name为null表示清空整个地点）
    store.sendBatchItemDeletion([{ place_name: placeName }], false)
    ElMessage.success('清空请求已发送')
  } catch {
    // 用户取消操作
  }
}

// 清空全场所有物品
const handleClearAllItems = async () => {
  try {
    await ElMessageBox.confirm(
      '警告：此操作将删除所有地点中的所有物品，不可恢复！确定继续吗？',
      '危险操作',
      {
        confirmButtonText: '确定清空',
        cancelButtonText: '取消',
        type: 'error',
        distinguishCancelAndClose: true
      }
    )
    
    // 发送清空全场请求
    store.sendBatchItemDeletion([], true)
    ElMessage.success('全场清空请求已发送')
  } catch {
    // 用户取消操作
  }
}
</script>

<style scoped>
.place-status-card {
  margin-bottom: 20px;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.card-header h3 {
  margin: 0;
  color: #303133;
}

.place-status-content {
  padding: 10px 0;
}

.players-list {
  display: flex;
  flex-wrap: wrap;
  gap: 5px;
}

.player-tag {
  margin: 2px 0;
}

.items-list {
  display: flex;
  flex-direction: column;
  gap: 5px;
}

.item-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 4px 8px;
  background-color: #f5f7fa;
  border-radius: 4px;
}

.item-name {
  flex: 1;
  font-size: 13px;
  color: #606266;
}

.empty-text {
  color: #909399;
  font-size: 12px;
  font-style: italic;
}

.clear-place-btn {
  margin-top: 5px;
  width: 100%;
}
</style>