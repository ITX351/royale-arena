<template>
  <el-card class="place-status-card">
    <template #header>
      <div class="card-header">
        <h3>地点状态管理</h3>
        <div class="header-actions">
          <div class="button-group">
            <el-button 
              type="danger" 
              size="small" 
              @click="handleClearAllItems"
              :disabled="!hasAnyItems"
            >
              清空全场物品
            </el-button>
            <el-button 
              type="primary" 
              size="small"
              @click="showPlainTextDialog('place')"
            >
              复制状态
            </el-button>
          </div>
          <el-button 
            type="primary" 
            size="small" 
            @click="isCollapsed = !isCollapsed"
            :icon="isCollapsed ? ArrowDown : ArrowUp"
            circle
          />
        </div>
      </div>
    </template>
    <el-collapse-transition>
      <div v-show="!isCollapsed" class="place-status-content">
        <el-table :data="placeList" style="width: 100%" size="small" max-height="500">
          <el-table-column prop="name" label="地点名称" width="120" />
          <el-table-column label="状态" width="120">
            <template #default="scope">
              <el-switch
                v-model="scope.row.is_destroyed"
                @change="handlePlaceStatusChange(scope.row.name, $event)"
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
                <div class="items-flow">
                  <el-tag
                    v-for="(item, index) in scope.row.items"
                    :key="index"
                    size="small"
                    class="item-tag"
                    closable
                    @close="handleDeleteItem(scope.row.name, item.name)"
                  >
                    {{ item.name }}
                  </el-tag>
                  <span v-if="scope.row.items.length === 0" class="empty-text">无</span>
                </div>
                <el-button
                  v-if="scope.row.items.length > 0"
                  type="warning"
                  size="small"
                  circle
                  :icon="Delete"
                  class="clear-place-btn"
                  @click="handleClearPlaceItems(scope.row.name)"
                />
              </div>
            </template>
          </el-table-column>
        </el-table>
      </div>
    </el-collapse-transition>
    
    <!-- 纯文本显示对话框 -->
    <el-dialog v-model="plainTextDialogVisible" :title="dialogTitle" width="600px">
      <el-input 
        type="textarea" 
        v-model="plainTextContent" 
        :rows="10" 
        readonly
      />
      <template #footer>
        <span class="dialog-footer">
          <el-button @click="plainTextDialogVisible = false">关闭</el-button>
          <el-button 
            type="primary" 
            @click="copyPlainTextContent"
          >
            复制到剪贴板
          </el-button>
        </span>
      </template>
    </el-dialog>
  </el-card>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { ElMessageBox, ElMessage } from 'element-plus'
import { Delete, ArrowUp, ArrowDown } from '@element-plus/icons-vue'
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

// 折叠状态
const isCollapsed = ref(true)

// 纯文本对话框相关
const plainTextDialogVisible = ref(false)
const plainTextContent = ref('')
const dialogTitle = ref('')

// 计算属性
const placeList = computed<Place[]>(() => {
  return props.places
})

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
  // 确保isDestroyed是布尔值
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

// 显示纯文本对话框
const showPlainTextDialog = (type: 'place' | 'player') => {
  if (type === 'place') {
    // 创建地点状态的表格文本表示
    let statusText = '地点\t玩家\t物品\n'
    statusText += '----\t----\t----\n'

    placeList.value
      .filter(place => !place.is_destroyed)
      .forEach(place => {
        const playerNames = place.players
          .map(playerId => getPlayerName(playerId))
          .join(', ') || '无'
        const itemNames = place.items
          .map(item => item.name)
          .join(', ') || '无'

        statusText += `${place.name}\t${playerNames}\t${itemNames}\n`
      })

    plainTextContent.value = statusText
    dialogTitle.value = '地点状态'
  }
  
  plainTextDialogVisible.value = true
}

// 复制纯文本内容到剪贴板
const copyPlainTextContent = () => {
  navigator.clipboard.writeText(plainTextContent.value).then(() => {
    ElMessage.success('内容已复制到剪贴板')
  }).catch(() => {
    ElMessage.error('复制失败')
  })
}
</script>

<style scoped>
.place-status-card {
  margin-bottom: 20px;
  width: 100%;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
}

.card-header h3 {
  margin: 0;
  color: #303133;
}

.header-actions {
  display: flex;
  gap: 10px;
  align-items: center;
}

.button-group {
  display: flex;
  gap: 10px;
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
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}

.items-flow {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  align-items: center;
  flex: 1;
}

.item-tag {
  margin: 2px 0;
}

.empty-text {
  color: #909399;
  font-size: 12px;
  font-style: italic;
}

.clear-place-btn {
  margin-left: auto;
}

.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
}
</style>