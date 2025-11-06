<template>
  <el-card
    class="player-status-card collapsible-card"
    :class="{ 'collapsible-card--collapsed': isCollapsed }"
  >
    <template #header>
      <div class="card-header">
        <h3>玩家状态管理</h3>
        <div class="header-actions">
          <el-checkbox 
            v-model="showDeadPlayers"
                  size="small"
          >
            显示已出局玩家
          </el-checkbox>
          <el-button 
            type="primary" 
            size="small" 
            @click="showPlainTextDialog('player')"
                >
            复制状态
          </el-button>
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
      <div v-show="!isCollapsed" class="player-status-content">
        <el-table
          :data="sortedPlayers"
          style="width: 100%"
          size="small"
          max-height="400"
          :fit="false"
        >
          <el-table-column label="玩家" min-width="100">
            <template #header>
              <div
                class="sortable-header"
                role="button"
                tabindex="0"
                @click="toggleSort('name')"
                @keydown.enter.prevent="toggleSort('name')"
                @keydown.space.prevent="toggleSort('name')"
              >
                玩家
                <ArrowUp v-if="sortKey === 'name' && sortOrder === 'asc'" class="sort-icon" />
                <ArrowDown v-else-if="sortKey === 'name' && sortOrder === 'desc'" class="sort-icon" />
              </div>
            </template>
            <template #default="scope">
              <div class="player-name-cell">
                <el-tooltip
                  effect="dark"
                  :content="scope.row.password ? `密码：${scope.row.password}` : '暂无密码'"
                  placement="right"
                >
                  <span
                    class="player-name"
                    role="link"
                    tabindex="0"
                    @click="goToActorPage(scope.row.password)"
                    @keydown.enter.prevent="goToActorPage(scope.row.password)"
                  >
                    {{ scope.row.name }}
                  </span>
                </el-tooltip>
              </div>
            </template>
          </el-table-column>
          <el-table-column label="票数" min-width="60">
            <template #header>
              <div
                class="sortable-header"
                role="button"
                tabindex="0"
                @click="toggleSort('votes')"
                @keydown.enter.prevent="toggleSort('votes')"
                @keydown.space.prevent="toggleSort('votes')"
              >
                票数
                <ArrowUp v-if="sortKey === 'votes' && sortOrder === 'asc'" class="sort-icon" />
                <ArrowDown v-else-if="sortKey === 'votes' && sortOrder === 'desc'" class="sort-icon" />
              </div>
            </template>
            <template #default="scope">
              <div class="status-value">
                {{ calculatePlayerVotes(scope.row) }}
              </div>
            </template>
          </el-table-column>
          <el-table-column label="位置" min-width="80" prop="location">
            <template #header>
              <div
                class="sortable-header"
                role="button"
                tabindex="0"
                @click="toggleSort('location')"
                @keydown.enter.prevent="toggleSort('location')"
                @keydown.space.prevent="toggleSort('location')"
              >
                位置
                <ArrowUp v-if="sortKey === 'location' && sortOrder === 'asc'" class="sort-icon" />
                <ArrowDown v-else-if="sortKey === 'location' && sortOrder === 'desc'" class="sort-icon" />
              </div>
            </template>
          </el-table-column>
          <el-table-column label="生命" min-width="60">
            <template #header>
              <div
                class="sortable-header"
                role="button"
                tabindex="0"
                @click="toggleSort('life')"
                @keydown.enter.prevent="toggleSort('life')"
                @keydown.space.prevent="toggleSort('life')"
              >
                生命
                <ArrowUp v-if="sortKey === 'life' && sortOrder === 'asc'" class="sort-icon" />
                <ArrowDown v-else-if="sortKey === 'life' && sortOrder === 'desc'" class="sort-icon" />
              </div>
            </template>
            <template #default="scope">
              <div class="status-value">
                <el-input 
                  v-model="scope.row.life"
                  @focus="() => handleEditableFieldFocus(scope.row, 'life')"
                  @blur="(event: FocusEvent) => handleLifeBlur(scope.row, event)"
                  size="small"
                />
              </div>
            </template>
          </el-table-column>
          <el-table-column label="体力" min-width="60">
            <template #header>
              <div
                class="sortable-header"
                role="button"
                tabindex="0"
                @click="toggleSort('strength')"
                @keydown.enter.prevent="toggleSort('strength')"
                @keydown.space.prevent="toggleSort('strength')"
              >
                体力
                <ArrowUp v-if="sortKey === 'strength' && sortOrder === 'asc'" class="sort-icon" />
                <ArrowDown v-else-if="sortKey === 'strength' && sortOrder === 'desc'" class="sort-icon" />
              </div>
            </template>
            <template #default="scope">
              <div class="status-value">
                <el-input 
                  v-model="scope.row.strength"
                  @focus="() => handleEditableFieldFocus(scope.row, 'strength')"
                  @blur="(event: FocusEvent) => handleStrengthBlur(scope.row, event)"
                  size="small"
                />
              </div>
            </template>
          </el-table-column>
          <el-table-column label="物品" min-width="240">
            <template #default="scope">
              <div class="items-container">
                <el-button 
                  type="success" 
                  size="small"
                  circle
                  :icon="Plus"
                  aria-label="添加物品"
                  @click="showAddItemDialog(scope.row.id)"
                />
                <el-tag
                  v-if="scope.row.equipped_weapon"
                  effect="dark"
                  type="danger"
                  size="small"
                  class="equipment-tag weapon-tag"
                  closable
                  @close="() => scope.row.equipped_weapon && removeItem(scope.row.id, scope.row.equipped_weapon.name)"
                >
                  {{ getItemDisplayName(scope.row.equipped_weapon) }}
                </el-tag>
                <el-tag
                  v-if="scope.row.equipped_armor"
                  effect="dark"
                  type="info"
                  size="small"
                  class="equipment-tag armor-tag"
                  closable
                  @close="() => scope.row.equipped_armor && removeItem(scope.row.id, scope.row.equipped_armor.name)"
                >
                  {{ getItemDisplayName(scope.row.equipped_armor) }}
                </el-tag>
                <el-tag 
                  v-for="(item, index) in scope.row.inventory" 
                  :key="index" 
                  size="small" 
                  class="item-tag"
                  closable
                  @close="removeItem(scope.row.id, item.name)"
                >
                  {{ getItemDisplayName(item) }}
                </el-tag>
              </div>
            </template>
          </el-table-column>
          <el-table-column label="操作" min-width="80">
            <template #default="scope">
              <el-button 
                size="small" 
                :type="scope.row.is_bound ? 'warning' : 'primary'"
                @click="togglePlayerBinding(scope.row.id)"
              >
                {{ scope.row.is_bound ? '松绑' : '捆绑' }}
              </el-button>
            </template>
          </el-table-column>
        </el-table>
      </div>
    </el-collapse-transition>
    
    <!-- 添加物品对话框 -->
    <ItemSelectionDialog
      v-model="addItemDialogVisible"
      title="添加物品"
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
  </el-card>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { ElMessage } from 'element-plus'
import { ArrowUp, ArrowDown, Plus } from '@element-plus/icons-vue'
import { useGameStateStore } from '@/stores/gameState'
import ItemSelectionDialog from '@/components/common/ItemSelectionDialog.vue'
import { calculatePlayerVotes } from '@/utils/playerUtils'
import { getItemDisplayName } from '@/utils/itemDisplay'
import type { Player } from '@/types/gameStateTypes'

// 定义组件属性
const props = defineProps<{
  players: Player[]
}>()

// 定义事件发射
const emit = defineEmits<{
  (e: 'player-binding-change', playerId: string): void
}>()

const store = useGameStateStore()
const router = useRouter()
const route = useRoute()

// 折叠状态，默认展开
const isCollapsed = ref(false)

// 添加物品对话框相关
const addItemDialogVisible = ref(false)
const addItemForm = ref({
  playerId: '',
  itemName: ''
})

// 纯文本对话框相关
const plainTextDialogVisible = ref(false)
const plainTextContent = ref('')
const dialogTitle = ref('')

// 计算属性
const playerList = computed<Player[]>(() => {
  return props.players
})

const showDeadPlayers = ref(true)

type SortKey = 'name' | 'votes' | 'location' | 'life' | 'strength'
type SortOrder = 'asc' | 'desc'

const sortKey = ref<SortKey>('name')
const sortOrder = ref<SortOrder>('asc')
const sortLockOrder = ref<string[] | null>(null)
const editingSortKey = ref<SortKey | null>(null)

interface EditingState {
  playerId: string
  key: SortKey
  originalValue: number
}

const editingState = ref<EditingState | null>(null)

const filteredPlayers = computed<Player[]>(() => {
  if (showDeadPlayers.value) {
    return playerList.value
  }
  return playerList.value.filter(player => player.is_alive)
})

const getSortValue = (player: Player, key: SortKey): string | number => {
  switch (key) {
    case 'votes':
      return calculatePlayerVotes(player)
    case 'location':
      return player.location
    case 'life':
      return player.life
    case 'strength':
      return player.strength
    case 'name':
    default:
      return player.name
  }
}

const computeSortedPlayers = (): Player[] => {
  const alivePlayers = filteredPlayers.value.filter(player => player.is_alive)
  const eliminatedPlayers = filteredPlayers.value.filter(player => !player.is_alive)

  const direction = sortOrder.value === 'asc' ? 1 : -1

  const comparePlayers = (a: Player, b: Player) => {
    const aVal = getSortValue(a, sortKey.value)
    const bVal = getSortValue(b, sortKey.value)

    let result: number

    if (typeof aVal === 'number' && typeof bVal === 'number') {
      result = aVal - bVal
    } else {
      result = String(aVal).localeCompare(String(bVal))
    }

    if (result === 0) {
      result = a.name.localeCompare(b.name)
    }

    return result * direction
  }

  alivePlayers.sort(comparePlayers)
  eliminatedPlayers.sort(comparePlayers)

  return [...alivePlayers, ...eliminatedPlayers]
}

const baseSortedPlayers = computed<Player[]>(() => computeSortedPlayers())

const sortedPlayers = computed<Player[]>(() => {
  const base = baseSortedPlayers.value

  if (!sortLockOrder.value || editingSortKey.value !== sortKey.value) {
    return base
  }

  const orderMap = new Map<string, number>()
  sortLockOrder.value.forEach((id, index) => {
    orderMap.set(id, index)
  })

  return [...base].sort((a, b) => {
    const aIndex = orderMap.get(a.id)
    const bIndex = orderMap.get(b.id)

    if (aIndex != null && bIndex != null) {
      return aIndex - bIndex
    }

    if (aIndex != null) {
      return -1
    }

    if (bIndex != null) {
      return 1
    }

    return base.indexOf(a) - base.indexOf(b)
  })
})

const lockSortOrder = (key: SortKey) => {
  if (sortLockOrder.value || sortKey.value !== key) {
    return
  }

  sortLockOrder.value = sortedPlayers.value.map(player => player.id)
  editingSortKey.value = key
}

const unlockSortOrder = () => {
  sortLockOrder.value = null
  editingSortKey.value = null
}

const toggleSort = (key: SortKey) => {
  unlockSortOrder()

  if (sortKey.value === key) {
    sortOrder.value = sortOrder.value === 'asc' ? 'desc' : 'asc'
  } else {
    sortKey.value = key
    sortOrder.value = 'asc'
  }
}

const handleEditableFieldFocus = (player: Player, key: SortKey) => {
  const originalValue = key === 'life' ? player.life : player.strength
  editingState.value = {
    playerId: player.id,
    key,
    originalValue
  }

  lockSortOrder(key)
}

const finishEditing = () => {
  editingState.value = null
  unlockSortOrder()
}

const handleLifeBlur = (player: Player, event: FocusEvent) => {
  const newValueStr = (event.target as HTMLInputElement).value
  const currentValue = editingState.value && editingState.value.playerId === player.id && editingState.value.key === 'life'
    ? editingState.value.originalValue
    : player.life

  updatePlayerLife(player.id, currentValue, newValueStr)
  finishEditing()
}

const handleStrengthBlur = (player: Player, event: FocusEvent) => {
  const newValueStr = (event.target as HTMLInputElement).value
  const currentValue = editingState.value && editingState.value.playerId === player.id && editingState.value.key === 'strength'
    ? editingState.value.originalValue
    : player.strength

  updatePlayerStrength(player.id, currentValue, newValueStr)
  finishEditing()
}

// 玩家状态管理方法
const togglePlayerBinding = (playerId: string) => {
  // 调用store中的方法处理玩家捆绑/松绑
  store.togglePlayerBinding(playerId)
  // 发送事件通知父组件
  emit('player-binding-change', playerId)
}

// 更新玩家生命值
const updatePlayerLife = (playerId: string, currentValue: number, newValueStr: string) => {
  const newValue = parseInt(newValueStr, 10)
  // 只有当值发生变化时才提交修改
  if (!isNaN(newValue) && newValue !== currentValue) {
    store.setPlayerLife(playerId, newValue)
  }
}

// 更新玩家体力值
const updatePlayerStrength = (playerId: string, currentValue: number, newValueStr: string) => {
  const newValue = parseInt(newValueStr, 10)
  // 只有当值发生变化时才提交修改
  if (!isNaN(newValue) && newValue !== currentValue) {
    store.setPlayerStrength(playerId, newValue)
  }
}

// 显示添加物品对话框
const showAddItemDialog = (playerId: string) => {
  addItemForm.value.playerId = playerId
  addItemForm.value.itemName = ''
  addItemDialogVisible.value = true
}

// 处理添加物品确认
const handleAddItemConfirm = (itemName: string) => {
  if (!itemName) {
    ElMessage.error('请选择物品')
    return
  }
  
  // 调用store方法添加物品（使用物品名称而不是完整对象）
  store.addPlayerItem(addItemForm.value.playerId, itemName)
  ElMessage.success('物品已添加')
}

// 处理添加物品取消
const handleAddItemCancel = () => {
  // 保持对话框关闭状态，无需额外操作
}

// 移除物品
const removeItem = (playerId: string, itemName: string) => {
  store.removePlayerItem(playerId, itemName)
  ElMessage.success('物品已移除')
}

// 显示纯文本对话框
const showPlainTextDialog = (type: 'place' | 'player') => {
  if (type === 'player') {
    // 创建玩家状态的表格文本表示（包含票数列）
    let statusText = '玩家\t票数\t位置\t生命值\t体力值\t物品\n'
    // statusText += '----\t----\t----\t------\t------\t----\n'

    playerList.value
      .filter(player => player.is_alive)
      .forEach(player => {
      const items = player.inventory.map(item => getItemDisplayName(item)).join(', ')
      const votes = calculatePlayerVotes(player)
      statusText += `${player.name}\t${votes}\t${player.location}\t${player.life}\t${player.strength}\t${items || '无'}\n`
      })

    plainTextContent.value = statusText
    dialogTitle.value = '玩家状态'
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

// 跳转到演员界面
const goToActorPage = (playerPassword: string) => {
  const currentGameId = route.params.id as string | undefined

  if (!currentGameId) {
    ElMessage.error('无法确定当前游戏信息')
    return
  }

  if (!playerPassword) {
    ElMessage.warning('该玩家尚未设置密码')
    return
  }

  const actorRoute = router.resolve({
    name: 'ActorMainWithPassword',
    params: {
      id: currentGameId,
      password: playerPassword
    }
  })

  window.open(actorRoute.href, '_blank', 'noopener')
}
</script>

<style scoped>
.player-status-card {
  margin-bottom: 20px;
  width: 100%;
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

.header-actions {
  display: flex;
  gap: 10px;
  align-items: center;
}

.player-status-content {
  padding: 10px 0;
}

.status-value {
  display: flex;
  align-items: center;
  gap: 5px;
}

.items-container {
  display: flex;
  flex-wrap: wrap;
  gap: 5px;
  align-items: center;
}

.item-tag {
  margin: 2px 0;
}

.equipment-tag {
  font-weight: 600;
}

.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
}

.player-name-cell {
  display: flex;
  align-items: center;
  gap: 6px;
}

.player-name {
  cursor: pointer;
  color: #409eff;
}

.player-name:focus {
  outline: none;
  text-decoration: underline;
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