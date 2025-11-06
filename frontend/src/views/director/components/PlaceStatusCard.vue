<template>
  <el-card
    class="place-status-card collapsible-card"
    :class="{ 'collapsible-card--collapsed': isCollapsed }"
  >
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
              清空物品
            </el-button>
            <el-button 
              type="success" 
              size="small"
              @click="openBatchAirdropDialog"
            >
              批量空投
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
          <el-table-column label="地点名称" min-width="140">
            <template #header>
              <div
                class="sortable-header"
                role="button"
                tabindex="0"
                @click="togglePlaceSort"
                @keydown.enter.prevent="togglePlaceSort"
                @keydown.space.prevent="togglePlaceSort"
              >
                地点名称
                <ArrowUp v-if="placeSortOrder === 'asc'" class="sort-icon" />
                <ArrowDown v-else class="sort-icon" />
              </div>
            </template>
            <template #default="scope">
              {{ scope.row.name }}
            </template>
          </el-table-column>
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
              <div class="items-container">
                <el-button 
                  type="success" 
                  size="small"
                  circle
                  :icon="Plus"
                  aria-label="添加物品"
                  @click="showAddItemDialog(scope.row.name)"
                />
                <div class="items-flow">
                  <el-tag
                    v-for="(item, index) in scope.row.items"
                    :key="index"
                    size="small"
                    class="item-tag"
                    closable
                    @close="handleDeleteItem(scope.row.name, item.name)"
                  >
                    {{ getItemDisplayName(item) }}
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
    
    <!-- 添加物品对话框 -->
    <ItemSelectionDialog
      v-model="addItemDialogVisible"
      title="向地点添加物品"
      item-label="物品名称"
      placeholder="请选择物品"
      width="400px"
      :initial-selected-item="addItemForm.itemName"
      @confirm="handleAddItemConfirm"
      @cancel="handleAddItemCancel"
    />
    
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
    
    <!-- 批量空投对话框 -->
    <BatchAirdropDialog 
      v-if="showBatchDialog"
      v-model="showBatchDialog"
      :rules-json="rulesJson"
      :existing-items="existingItems"
      :available-places="availablePlaces"
      @confirm="handleBatchAirdrop"
    />
  </el-card>
</template>

<script setup lang="ts">
import { ref, computed, defineAsyncComponent } from 'vue'
import { ElMessageBox, ElMessage } from 'element-plus'
import { Delete, Plus, ArrowUp, ArrowDown } from '@element-plus/icons-vue'
import { useGameStateStore } from '@/stores/gameState'
import ItemSelectionDialog from '@/components/common/ItemSelectionDialog.vue'
import { extractExistingWeaponsAndArmorsFromGameState } from '@/utils/itemParser'
import { getItemDisplayName } from '@/utils/itemDisplay'
import type { DirectorPlace as Place, DirectorGameData } from '@/types/gameStateTypes'

// 异步加载批量空投对话框组件
const BatchAirdropDialog = defineAsyncComponent(() => import('./BatchAirdropDialog.vue'))

// 定义组件属性
const props = defineProps<{
  places: Place[]
}>()

// 定义事件发射
const emit = defineEmits<{
  (e: 'place-status-change', placeName: string, isDestroyed: boolean): void
}>()

const store = useGameStateStore()

type SortOrder = 'asc' | 'desc'

const placeSortOrder = ref<SortOrder>('asc')

// 初始自动折叠状态
const isCollapsed = ref(false)

// 添加物品对话框相关
const addItemDialogVisible = ref(false)
const addItemForm = ref({
  placeName: '',
  itemName: ''
})

// 纯文本对话框相关
const plainTextDialogVisible = ref(false)
const plainTextContent = ref('')
const dialogTitle = ref('')

// 批量空投对话框相关
const showBatchDialog = ref(false)

// 计算属性
const placeList = computed<Place[]>(() => {
  const direction = placeSortOrder.value === 'asc' ? 1 : -1

  return [...props.places].sort((a, b) => {
    if (a.is_destroyed === b.is_destroyed) {
      return a.name.localeCompare(b.name) * direction
    }
    return a.is_destroyed ? 1 : -1
  })
})

const hasAnyItems = computed(() => {
  return placeList.value.some(place => place.items.length > 0)
})

// 批量空投相关的计算属性
const rulesJson = computed(() => {
  return store.gameState?.global_state?.rules_config || {}
})

const existingItems = computed(() => {
  // 从游戏状态中提取现有物品
  return extractExistingWeaponsAndArmorsFromGameState(store.gameData as DirectorGameData)
})

const availablePlaces = computed(() => {
  if (!props.places) return []
  return props.places
    .filter(place => !place.is_destroyed)
    .map(place => place.name)
})

const togglePlaceSort = () => {
  placeSortOrder.value = placeSortOrder.value === 'asc' ? 'desc' : 'asc'
}

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
const handleDeleteItem = (placeName: string, itemName: string) => {
  // 发送删除请求
  store.sendBatchItemDeletion([{ place_name: placeName, item_name: itemName }], false)
  ElMessage.success('删除请求已发送')
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

// 显示添加物品对话框
const showAddItemDialog = (placeName: string) => {
  addItemForm.value.placeName = placeName
  addItemForm.value.itemName = ''
  addItemDialogVisible.value = true
}

// 处理添加物品确认
const handleAddItemConfirm = (itemName: string) => {
  if (!itemName) {
    ElMessage.error('请选择物品')
    return
  }
  
  // 调用store方法添加物品到地点（使用后端空投接口）
  store.sendBatchAirdrop([
    {
      item_name: itemName,
      place_name: addItemForm.value.placeName
    }
  ])
  ElMessage.success('物品已添加到地点')
}

// 处理添加物品取消
const handleAddItemCancel = () => {
  // 保持对话框关闭状态，无需额外操作
}

// 显示纯文本对话框
const showPlainTextDialog = (type: 'place' | 'player') => {
  if (type === 'place') {
    // 创建地点状态的表格文本表示
    let statusText = '地点\t玩家\t物品\n'
    // statusText += '----\t----\t----\n'

    placeList.value
      .filter(place => !place.is_destroyed)
      .forEach(place => {
        const playerNames = place.players
          .map(playerId => getPlayerName(playerId))
          .join(', ') || '无'
        const itemNames = place.items
          .map(item => getItemDisplayName(item))
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

// 批量空投相关方法
const openBatchAirdropDialog = () => {
  showBatchDialog.value = true
}

const handleBatchAirdrop = (airdrops: Array<{ item_name: string, place_name: string }>) => {
  store.sendBatchAirdrop(airdrops)
  ElMessage.success(`批量空投已发送，共 ${airdrops.length} 个物品`)
  
  showBatchDialog.value = false
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

.items-container {
  display: flex;
  flex-wrap: wrap;
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

.sortable-header {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  cursor: pointer;
  user-select: none;
}

.sortable-header:focus-visible {
  outline: 2px solid #409eff;
  border-radius: 2px;
}

.sort-icon {
  width: 14px;
  height: 14px;
}
</style>