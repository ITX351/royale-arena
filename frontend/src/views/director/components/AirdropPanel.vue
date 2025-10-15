<template>
  <el-card class="airdrop-panel">
    <template #header>
      <div class="card-header">
        <h3>空投设置</h3>
        <el-button 
          type="primary" 
          size="small" 
          @click="isCollapsed = !isCollapsed"
          :icon="isCollapsed ? ArrowDown : ArrowUp"
          circle
        />
      </div>
    </template>

    <el-collapse-transition>
      <div v-show="!isCollapsed" class="airdrop-content">
        <!-- 单次空投区域 -->
        <div class="single-airdrop-section">
          <h4>单次空投</h4>
          <el-form :model="singleAirdropForm" ref="singleAirdropFormRef">
            <el-form-item label="选择物品">
              <el-select 
                v-model="singleAirdropForm.selectedItem" 
                placeholder="请选择物品"
                style="width: 100%"
                filterable
              >
                <el-option
                  v-for="item in allItemOptions"
                  :key="item"
                  :label="item"
                  :value="item"
                />
              </el-select>
            </el-form-item>
            <el-form-item label="选择地点">
              <el-select 
                v-model="singleAirdropForm.selectedPlace" 
                placeholder="请选择地点"
                style="width: 100%"
              >
                <el-option
                  v-for="place in availablePlaces"
                  :key="place"
                  :label="place"
                  :value="place"
                />
              </el-select>
            </el-form-item>
            <el-form-item>
              <el-button 
                type="primary" 
                @click="executeSingleAirdrop"
                :disabled="!singleAirdropForm.selectedItem || !singleAirdropForm.selectedPlace"
                :loading="executing"
              >
                确认空投
              </el-button>
            </el-form-item>
          </el-form>
        </div>

        <el-divider />

        <!-- 批量空投区域 -->
        <div class="batch-airdrop-section">
          <h4>批量空投</h4>
          <el-button 
            type="success" 
            @click="openBatchAirdropDialog"
          >
            打开批量空投界面
          </el-button>
        </div>
      </div>
    </el-collapse-transition>

    <!-- 批量空投对话框 -->
    <BatchAirdropDialog 
      v-if="BatchAirdropDialog"
      v-model="showBatchDialog"
      :rules-json="rulesJson"
      :existing-items="existingItems"
      :available-places="availablePlaces"
      @confirm="handleBatchAirdrop"
    />
  </el-card>
</template>

<script setup lang="ts">
import { ref, reactive, computed, defineAsyncComponent } from 'vue'
import { ElMessage } from 'element-plus'
import { ArrowUp, ArrowDown } from '@element-plus/icons-vue'
import { useGameStateStore } from '@/stores/gameState'
import { createItemParser, extractExistingItemsFromGameState } from '@/utils/itemParser'
import type { DirectorGameData } from '@/types/gameStateTypes'

// 异步加载批量空投对话框组件
const BatchAirdropDialog = defineAsyncComponent(() => import('./BatchAirdropDialog.vue'))

// 定义组件属性
const props = defineProps<{
  gameId: string
}>()

// 定义事件发射
const emit = defineEmits<{
  (e: 'airdrop-accepted', items: any[], place: string): void
}>()

// 使用store
const store = useGameStateStore()

// 折叠状态，默认折叠
const isCollapsed = ref(true)

// 响应式状态
const singleAirdropFormRef = ref()
const singleAirdropForm = reactive({
  selectedItem: '',
  selectedPlace: ''
})

const showBatchDialog = ref(false)
const executing = ref(false)

// 计算属性
const rulesJson = computed(() => {
  return store.gameState?.global_state?.rules_config || {}
})

const existingItems = computed(() => {
  return extractExistingItemsFromGameState(store.gameData as DirectorGameData)
})

const availablePlaces = computed(() => {
  if (!store.directorPlaces) return []
  return Object.values(store.directorPlaces)
    .filter(place => !place.is_destroyed)
    .map(place => place.name)
})

const allItemOptions = computed(() => {
  if (!rulesJson.value.items) return []
  
  try {
    const parser = createItemParser(rulesJson.value, existingItems.value)
    const parsedItems = parser.parseAllItems()
    return parsedItems.allItems
  } catch (error) {
    console.error('解析物品列表失败:', error)
    return []
  }
})

// 方法实现
const executeSingleAirdrop = async () => {
  if (!singleAirdropForm.selectedItem || !singleAirdropForm.selectedPlace) {
    ElMessage.warning('请选择物品和地点')
    return
  }
  
  executing.value = true
  
  try {
    // 调用批量空投接口，但只传一个物品
    store.sendBatchAirdrop([
      {
        item_name: singleAirdropForm.selectedItem,
        place_name: singleAirdropForm.selectedPlace
      }
    ])
    
    ElMessage.success('单次空投已发送')
    
    // 发射事件给父组件
    emit('airdrop-accepted', [{ name: singleAirdropForm.selectedItem }], singleAirdropForm.selectedPlace)
    
    // 清空表单
    singleAirdropForm.selectedItem = ''
    singleAirdropForm.selectedPlace = ''
  } catch (error) {
    console.error('单次空投失败:', error)
    ElMessage.error('单次空投失败')
  } finally {
    executing.value = false
  }
}

const openBatchAirdropDialog = () => {
  showBatchDialog.value = true
}

const handleBatchAirdrop = (airdrops: Array<{ item_name: string, place_name: string }>) => {
  store.sendBatchAirdrop(airdrops)
  ElMessage.success(`批量空投已发送，共 ${airdrops.length} 个物品`)
  
  // 发射事件给父组件
  emit('airdrop-accepted', airdrops.map(a => ({ name: a.item_name })), '多个地点')
  
  showBatchDialog.value = false
}
</script>

<style scoped>
.airdrop-panel {
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

.airdrop-content {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.single-airdrop-section h4,
.batch-airdrop-section h4 {
  margin: 0 0 15px 0;
  color: #606266;
  font-size: 16px;
  font-weight: 600;
}

.single-airdrop-section {
  padding: 15px;
  background: #f8f9fa;
  border-radius: 6px;
  border: 1px solid #e1e6f0;
}

.batch-airdrop-section {
  text-align: center;
  padding: 20px;
}

@media (max-width: 768px) {
  .airdrop-content {
    gap: 15px;
  }
  
  .single-airdrop-section {
    padding: 10px;
  }
}
</style>