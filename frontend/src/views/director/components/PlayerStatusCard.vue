<template>
  <el-card class="player-status-card">
    <template #header>
      <div class="card-header">
        <h3>玩家状态管理</h3>
        <div class="header-actions">
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
        <el-table :data="playerList" style="width: 100%" size="small" max-height="400">
          <el-table-column prop="name" label="玩家" width="120" />
          <el-table-column label="生命值" width="150">
            <template #default="scope">
              <div class="status-value">
                <el-input 
                  v-model="scope.row.life"
                  @blur="(event: FocusEvent) => updatePlayerLife(scope.row.id, scope.row.life, (event.target as HTMLInputElement).value)"
                  size="small"
                />
              </div>
            </template>
          </el-table-column>
          <el-table-column label="体力值" width="150">
            <template #default="scope">
              <div class="status-value">
                <el-input 
                  v-model="scope.row.strength"
                  @blur="(event: FocusEvent) => updatePlayerStrength(scope.row.id, scope.row.strength, (event.target as HTMLInputElement).value)"
                  size="small"
                />
              </div>
            </template>
          </el-table-column>
          <el-table-column label="物品" min-width="200">
            <template #default="scope">
              <div class="items-container">
                <el-tag 
                  v-for="(item, index) in scope.row.inventory" 
                  :key="index" 
                  size="small" 
                  class="item-tag"
                  closable
                  @close="removeItem(scope.row.id, item.name)"
                >
                  {{ item.name }}
                </el-tag>
                <el-button 
                  type="success" 
                  size="small" 
                  @click="showAddItemDialog(scope.row.id)"
                >
                  添加物品
                </el-button>
              </div>
            </template>
          </el-table-column>
          <el-table-column label="操作" width="120">
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
    <el-dialog v-model="addItemDialogVisible" title="添加物品" width="400px">
      <el-form :model="addItemForm" label-width="80px">
        <el-form-item label="物品名称">
          <el-select 
            v-model="addItemForm.itemName" 
            placeholder="请选择物品" 
            filterable
            clearable
          >
            <el-option 
              v-for="item in allItemOptions" 
              :key="item" 
              :label="item" 
              :value="item"
            />
          </el-select>
        </el-form-item>
      </el-form>
      <template #footer>
        <span class="dialog-footer">
          <el-button @click="addItemDialogVisible = false">取消</el-button>
          <el-button type="primary" @click="confirmAddItem">确定</el-button>
        </span>
      </template>
    </el-dialog>
    
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
import { ElMessage } from 'element-plus'
import { ArrowUp, ArrowDown } from '@element-plus/icons-vue'
import { useGameStateStore } from '@/stores/gameState'
import { createItemParser, extractExistingItemsFromGameState } from '@/utils/itemParser'
import type { Player, DirectorGameData } from '@/types/gameStateTypes'

// 定义组件属性
const props = defineProps<{
  players: Player[]
}>()

// 定义事件发射
const emit = defineEmits<{
  (e: 'player-binding-change', playerId: string): void
}>()

const store = useGameStateStore()

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

// 复用AirdropPanel中的物品选项计算逻辑
const allItemOptions = computed(() => {
  const rulesJson = store.gameState?.global_state?.rules_config || {}
  if (!rulesJson.items) return []
  
  try {
    // 获取现有物品
    const existingItems = extractExistingItemsFromGameState(store.gameData as DirectorGameData)
    
    // 创建物品解析器
    const parser = createItemParser(rulesJson, existingItems)
    
    // 解析所有物品
    const parsedItems = parser.parseAllItems()
    return parsedItems.allItems
  } catch (error) {
    console.error('解析物品列表失败:', error)
    return []
  }
})

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

// 确认添加物品
const confirmAddItem = () => {
  if (!addItemForm.value.itemName) {
    ElMessage.error('请选择物品')
    return
  }
  
  // 调用store方法添加物品（使用物品名称而不是完整对象）
  store.addPlayerItem(addItemForm.value.playerId, addItemForm.value.itemName)
  ElMessage.success('物品已添加')
  
  // 关闭对话框
  addItemDialogVisible.value = false
}

// 移除物品
const removeItem = (playerId: string, itemName: string) => {
  store.removePlayerItem(playerId, itemName)
  ElMessage.success('物品已移除')
}

// 显示纯文本对话框
const showPlainTextDialog = (type: 'place' | 'player') => {
  if (type === 'player') {
    // 创建玩家状态的表格文本表示
    let statusText = '玩家\t生命值\t体力值\t物品\n'
    statusText += '----\t------\t------\t----\n'
    
    playerList.value.forEach(player => {
      const items = player.inventory.map(item => item.name).join(', ')
      statusText += `${player.name}\t${player.life}\t${player.strength}\t${items || '无'}\n`
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

.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
}
</style>