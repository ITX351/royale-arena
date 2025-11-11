<template>
  <el-dialog
    v-model="visible"
    title="批量空投设置"
    width="80%"
    :close-on-click-modal="false"
    :close-on-press-escape="false"
    @close="handleClose"
  >
    <div class="batch-airdrop-content">
      <!-- 稀有度选择区域 -->
      <div class="rarity-selection-section">
        <h4>武器和防具（按稀有度选择）</h4>
        <div class="rarity-grid">
          <div 
            v-for="option in weaponRarityOptions" 
            :key="option.rarityKey"
            class="rarity-option"
          >
            <el-form-item :label="option.displayName">
              <el-input-number
                v-model="raritySelections[option.rarityKey]"
                :min="0"
                :max="option.availableCount"
                :disabled="option.availableCount === 0"
                placeholder="数量"
                style="width: 100%"
              />
            </el-form-item>
          </div>
          <div class="rarity-break" aria-hidden="true"></div>
          <div 
            v-for="option in armorRarityOptions" 
            :key="option.rarityKey"
            class="rarity-option"
          >
            <el-form-item :label="option.displayName">
              <el-input-number
                v-model="raritySelections[option.rarityKey]"
                :min="0"
                :max="option.availableCount"
                :disabled="option.availableCount === 0"
                placeholder="数量"
                style="width: 100%"
              />
            </el-form-item>
          </div>
        </div>
      </div>

      <el-divider />

      <!-- 具体物品选择区域 -->
      <div class="specific-items-section">
        <h4>其他物品（具体名称选择）</h4>
        <div class="items-grid">
          <!-- 其他道具 -->
          <div class="item-category">
            <h5>功能道具</h5>
            <div v-for="item in parsedItems?.utilities || []" :key="item" class="item-option">
              <el-form-item :label="item">
                <el-input-number
                  v-model="specificSelections[item]"
                  :min="0"
                  placeholder="数量"
                  style="width: 100%"
                />
              </el-form-item>
            </div>
          </div>

          <!-- 消耗品 -->
          <div class="item-category">
            <h5>消耗品</h5>
            <div v-for="item in parsedItems?.consumables || []" :key="item" class="item-option">
              <el-form-item :label="item">
                <el-input-number
                  v-model="specificSelections[item]"
                  :min="0"
                  placeholder="数量"
                  style="width: 100%"
                />
              </el-form-item>
            </div>
          </div>

          <!-- 升级器 -->
          <div class="item-category">
            <h5>升级器</h5>
            <div v-for="item in parsedItems?.upgraders || []" :key="item" class="item-option">
              <el-form-item :label="item">
                <el-input-number
                  v-model="specificSelections[item]"
                  :min="0"
                  placeholder="数量"
                  style="width: 100%"
                />
              </el-form-item>
            </div>
          </div>
        </div>
      </div>

      <el-divider />

      <!-- 随机生成区域 -->
      <div class="generate-section">
        <el-button 
          type="success" 
          size="large"
          @click="generateRandomAssignment"
          :loading="generating"
          :disabled="!hasAnySelection"
        >
          随机生成地点分配
        </el-button>
      </div>

      <!-- 结果展示区域 -->
      <div v-if="generatedAirdrops.length > 0" class="result-section">
        <h4>空投分配结果</h4>
        
        <!-- 不足提醒 -->
        <el-alert
          v-if="insufficientWarnings.length > 0"
          :title="`以下稀有度物品数量不足：${insufficientWarnings.join('、')}`"
          type="warning"
          show-icon
          :closable="false"
          style="margin-bottom: 15px"
        />
        
        <!-- 结果表格 -->
        <el-table :data="generatedAirdrops" style="width: 100%" max-height="300">
          <el-table-column prop="item_name" label="物品名称" />
          <el-table-column label="投放地点" min-width="180">
            <template #default="{ row }">
              <el-select
                v-model="row.place_name"
                placeholder="选择地点"
                filterable
                :disabled="props.availablePlaces.length === 0"
                style="width: 100%"
              >
                <el-option
                  v-for="place in props.availablePlaces"
                  :key="place"
                  :label="place"
                  :value="place"
                />
              </el-select>
            </template>
          </el-table-column>
          <el-table-column label="操作" width="80">
            <template #default="{ $index }">
              <el-button 
                type="danger" 
                size="small" 
                @click="removeAirdropItem($index)"
              >
                移除
              </el-button>
            </template>
          </el-table-column>
        </el-table>
        <div v-if="placePreviews.length > 0" class="place-preview-section">
          <h5 class="place-preview-title">地点物品预览</h5>
          <p class="place-preview-hint">拖拽绿色标记的物品可调整投放地点</p>
          <div class="place-preview-grid">
            <div
              v-for="preview in placePreviews"
              :key="preview.name"
              class="place-preview-card"
            >
              <div class="place-preview-card__header">
                <span class="place-preview-card__name">{{ preview.name }}</span>
                <el-tag v-if="preview.isDestroyed" size="small" type="danger">已摧毁</el-tag>
              </div>
              <div class="place-preview-block">
                <span class="preview-label">已有物品</span>
                <div class="preview-tag-container">
                  <el-tag
                    v-for="(item, index) in preview.existing"
                    :key="`${preview.name}-existing-${index}`"
                    size="small"
                    class="existing-tag"
                  >
                    {{ item }}
                  </el-tag>
                  <span v-if="preview.existing.length === 0" class="preview-empty">无</span>
                </div>
              </div>
              <div
                class="place-preview-block place-preview-block--incoming"
                :class="{
                  'place-preview-block--active': dragOverPlace === preview.name,
                  'place-preview-block--disabled': !preview.canReceiveDrop
                }"
                @dragenter="handleIncomingDragEnter(preview, $event)"
                @dragover="handleIncomingDragOver(preview, $event)"
                @dragleave="handleIncomingDragLeave(preview, $event)"
                @drop="handleIncomingDrop(preview, $event)"
              >
                <span class="preview-label">即将投放</span>
                <div class="preview-tag-container">
                  <el-tag
                    v-for="incoming in preview.incoming"
                    :key="incoming.index"
                    size="small"
                    class="incoming-tag"
                    type="success"
                    draggable="true"
                    @dragstart="handleIncomingDragStart(incoming, $event)"
                    @dragend="handleIncomingDragEnd"
                  >
                    {{ incoming.itemName }}
                  </el-tag>
                  <span v-if="preview.incoming.length === 0" class="preview-empty">
                    {{ preview.canReceiveDrop ? '无' : '不可投放' }}
                  </span>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <template #footer>
      <span class="dialog-footer">
        <el-button @click="handleClose">取消</el-button>
        <el-button 
          @click="clearAll"
          :disabled="!hasAnySelection && generatedAirdrops.length === 0"
        >
          清空
        </el-button>
        <el-button 
          type="primary" 
          @click="handleConfirm"
          :disabled="generatedAirdrops.length === 0"
          :loading="confirming"
        >
          确认执行批量空投
        </el-button>
      </span>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { ref, reactive, computed, watch } from 'vue'
import { ElMessage } from 'element-plus'
import { createItemParser, type RarityOption, type ParsedItemInfo } from '@/utils/itemParser'
import { getItemDisplayName } from '@/utils/itemDisplay'
import type { DirectorPlace } from '@/types/gameStateTypes'

// 定义组件属性
const props = defineProps<{
  modelValue: boolean
  rulesJson: any
  existingItems: string[]
  availablePlaces: string[]
  places: DirectorPlace[]
}>()

// 定义事件发射
const emit = defineEmits<{
  'update:modelValue': [value: boolean]
  confirm: [airdrops: Array<{ item_name: string, place_name: string }>]
}>()

// 响应式状态
const visible = computed({
  get: () => props.modelValue,
  set: (value) => emit('update:modelValue', value)
})

const raritySelections = reactive<Record<string, number>>({})
const specificSelections = reactive<Record<string, number>>({})
const generatedAirdrops = ref<Array<{ item_name: string, place_name: string }>>([])
const insufficientWarnings = ref<string[]>([])
const generating = ref(false)
const confirming = ref(false)
const draggingItem = ref<{ index: number, itemName: string } | null>(null)
const dragOverPlace = ref<string | null>(null)

// 计算属性
const itemParser = computed(() => {
  return createItemParser(props.rulesJson, props.existingItems)
})

const parsedItems = computed<ParsedItemInfo | null>(() => {
  if (!itemParser.value) return null
  try {
    return itemParser.value.parseAllItems()
  } catch (error) {
    console.error('解析物品失败:', error)
    return null
  }
})

const batchAirdropOptions = computed(() => {
  if (!itemParser.value) return { weapons: [], armors: [] }
  try {
    return itemParser.value.getBatchAirdropRarityOptions()
  } catch (error) {
    console.error('获取批量空投选项失败:', error)
    return { weapons: [], armors: [] }
  }
})

const weaponRarityOptions = computed<RarityOption[]>(() => {
  return batchAirdropOptions.value.weapons
})

const armorRarityOptions = computed<RarityOption[]>(() => {
  return batchAirdropOptions.value.armors
})

const rarityOptions = computed<RarityOption[]>(() => {
  return [...batchAirdropOptions.value.weapons, ...batchAirdropOptions.value.armors]
})

const hasAnySelection = computed(() => {
  const hasRaritySelection = Object.values(raritySelections).some(count => count > 0)
  const hasSpecificSelection = Object.values(specificSelections).some(count => count > 0)
  return hasRaritySelection || hasSpecificSelection
})

interface IncomingPreview {
  itemName: string
  index: number
}

interface PlacePreview {
  name: string
  existing: string[]
  incoming: IncomingPreview[]
  isDestroyed: boolean
  canReceiveDrop: boolean
}

const availablePlaceSet = computed(() => new Set(props.availablePlaces))

const incomingGroups = computed(() => {
  const grouped = new Map<string, IncomingPreview[]>()
  generatedAirdrops.value.forEach((drop, index) => {
    const list = grouped.get(drop.place_name) ?? []
    list.push({ itemName: drop.item_name, index })
    grouped.set(drop.place_name, list)
  })
  return grouped
})

const placePreviews = computed<PlacePreview[]>(() => {
  return props.places.map(place => ({
    name: place.name,
    existing: (place.items || []).map(item => getItemDisplayName(item)),
    incoming: incomingGroups.value.get(place.name) ?? [],
    isDestroyed: place.is_destroyed,
    canReceiveDrop: availablePlaceSet.value.has(place.name)
  }))
})

// 监听器
watch(
  () => props.modelValue,
  (newValue) => {
    if (newValue) {
      // 对话框打开时初始化数据
      initializeSelections()
    } else {
      // 对话框关闭时清理数据
      clearGeneratedResults()
    }
  }
)

// 方法实现
const initializeSelections = () => {
  // 清空之前的选择
  Object.keys(raritySelections).forEach(key => {
    delete raritySelections[key]
  })
  Object.keys(specificSelections).forEach(key => {
    delete specificSelections[key]
  })
  
  // 初始化武器稀有度选择
  weaponRarityOptions.value.forEach(option => {
    raritySelections[option.rarityKey] = 0
  })
  // 初始化防具稀有度选择
  armorRarityOptions.value.forEach(option => {
    raritySelections[option.rarityKey] = 0
  })
  
  // 初始化具体物品选择
  if (parsedItems.value) {
  [...parsedItems.value.utilities, ...parsedItems.value.consumables, ...parsedItems.value.upgraders]
      .forEach(item => {
        specificSelections[item] = 0
      })
  }
}

const generateRandomAssignment = () => {
  if (!itemParser.value || !parsedItems.value) {
    ElMessage.error('物品解析器未初始化')
    return
  }
  
  if (props.availablePlaces.length === 0) {
    ElMessage.error('没有可用的地点')
    return
  }
  
  generating.value = true
  insufficientWarnings.value = []
  
  try {
    const selectedItems: string[] = []
    
    // 处理稀有度选择
    for (const [rarityKey, count] of Object.entries(raritySelections)) {
      if (count > 0) {
        const result = itemParser.value.pickItemsByRarity(rarityKey, count)
        selectedItems.push(...result.selectedItems)
        
        if (result.isInsufficient) {
          const option = rarityOptions.value.find(opt => opt.rarityKey === rarityKey)
          if (option) {
            insufficientWarnings.value.push(option.displayName)
          }
        }
      }
    }
    
    // 处理具体物品选择
    for (const [itemName, count] of Object.entries(specificSelections)) {
      if (count > 0) {
        for (let i = 0; i < count; i++) {
          selectedItems.push(itemName)
        }
      }
    }
    
    // 随机分配地点
    generatedAirdrops.value = itemParser.value.randomAssignPlaces(selectedItems, props.availablePlaces)
    
    ElMessage.success(`生成完成，共 ${generatedAirdrops.value.length} 个物品`)
  } catch (error) {
    console.error('生成随机分配失败:', error)
    ElMessage.error('生成失败')
  } finally {
    generating.value = false
  }
}

const removeAirdropItem = (index: number) => {
  generatedAirdrops.value.splice(index, 1)
}

const clearAll = () => {
  // 清空选择
  Object.keys(raritySelections).forEach(key => {
    raritySelections[key] = 0
  })
  Object.keys(specificSelections).forEach(key => {
    specificSelections[key] = 0
  })
  
  clearGeneratedResults()
  ElMessage.info('已清空所有选择')
}

const clearGeneratedResults = () => {
  generatedAirdrops.value = []
  insufficientWarnings.value = []
  draggingItem.value = null
  dragOverPlace.value = null
}

const handleConfirm = () => {
  if (generatedAirdrops.value.length === 0) {
    ElMessage.warning('请先生成空投分配')
    return
  }
  
  confirming.value = true
  
  try {
    emit('confirm', [...generatedAirdrops.value])
  } finally {
    confirming.value = false
  }
}

const handleClose = () => {
  emit('update:modelValue', false)
}

const handleIncomingDragStart = (incoming: IncomingPreview, event: DragEvent) => {
  draggingItem.value = { index: incoming.index, itemName: incoming.itemName }
  event.dataTransfer?.setData('text/plain', incoming.itemName)
  if (event.dataTransfer) {
    event.dataTransfer.effectAllowed = 'move'
  }
}

const handleIncomingDragEnd = () => {
  draggingItem.value = null
  dragOverPlace.value = null
}

const handleIncomingDragEnter = (preview: PlacePreview, event: DragEvent) => {
  if (!draggingItem.value || !preview.canReceiveDrop) return
  event.preventDefault()
  dragOverPlace.value = preview.name
}

const handleIncomingDragOver = (preview: PlacePreview, event: DragEvent) => {
  if (!draggingItem.value || !preview.canReceiveDrop) return
  event.preventDefault()
  if (event.dataTransfer) {
    event.dataTransfer.dropEffect = 'move'
  }
  dragOverPlace.value = preview.name
}

const handleIncomingDragLeave = (preview: PlacePreview, event: DragEvent) => {
  if (!preview.canReceiveDrop) return
  const currentTarget = event.currentTarget as HTMLElement | null
  const relatedTarget = event.relatedTarget as Node | null
  if (currentTarget && relatedTarget && currentTarget.contains(relatedTarget)) {
    return
  }
  if (dragOverPlace.value === preview.name) {
    dragOverPlace.value = null
  }
}

const handleIncomingDrop = (preview: PlacePreview, event: DragEvent) => {
  if (!draggingItem.value || !preview.canReceiveDrop) return
  event.preventDefault()
  const { index } = draggingItem.value
  if (generatedAirdrops.value[index]) {
    generatedAirdrops.value[index].place_name = preview.name
  }
  draggingItem.value = null
  dragOverPlace.value = null
}
</script>

<style scoped>
.batch-airdrop-content {
  display: flex;
  flex-direction: column;
  gap: 20px;
  max-height: 70vh;
  overflow-y: auto;
}

.rarity-selection-section h4,
.specific-items-section h4,
.result-section h4 {
  margin: 0 0 15px 0;
  color: #303133;
  font-size: 16px;
  font-weight: 600;
}

.rarity-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
  gap: 15px;
}

.rarity-option {
  padding: 10px;
  background: #f8f9fa;
  border-radius: 6px;
  border: 1px solid #e1e6f0;
}

.rarity-break {
  display: none;
}

.items-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 20px;
}

.item-category h5 {
  margin: 0 0 10px 0;
  color: #606266;
  font-size: 14px;
  font-weight: 600;
}

.item-option {
  margin-bottom: 10px;
}

.generate-section {
  text-align: center;
  padding: 20px;
}

.result-section {
  background: #f8f9fa;
  padding: 15px;
  border-radius: 6px;
  border: 1px solid #e1e6f0;
}

.place-preview-section {
  display: flex;
  flex-direction: column;
  gap: 10px;
  margin-top: 20px;
}

.place-preview-title {
  margin: 0;
  font-size: 15px;
  font-weight: 600;
  color: #303133;
}

.place-preview-hint {
  margin: 0;
  font-size: 12px;
  color: #909399;
}

.place-preview-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
  gap: 15px;
}

.place-preview-card {
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 12px;
  border: 1px solid #e1e6f0;
  border-radius: 6px;
  background: #ffffff;
  min-height: 150px;
}

.place-preview-card__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}

.place-preview-card__name {
  font-weight: 600;
  color: #303133;
}

.place-preview-block {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.preview-label {
  font-size: 12px;
  font-weight: 600;
  color: #606266;
}

.preview-tag-container {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  min-height: 26px;
}

.incoming-tag {
  background: #f0f9eb;
  color: #40916c;
  border-color: #c2e7b0;
  cursor: grab;
}

.incoming-tag:active {
  cursor: grabbing;
}

.place-preview-block--incoming {
  border: 1px dashed transparent;
  border-radius: 6px;
  padding: 10px;
  background: #f8fbf6;
  transition: border-color 0.2s ease, background-color 0.2s ease;
}

.place-preview-block--active {
  border-color: #67c23a;
  background: #f0f9eb;
}

.place-preview-block--disabled {
  background: #f5f5f5;
  border-color: #e4e7ed;
}

.preview-empty {
  color: #909399;
  font-size: 12px;
  font-style: italic;
}

.dialog-footer {
  display: flex;
  gap: 10px;
  justify-content: flex-end;
}

@media (max-width: 768px) {
  .rarity-grid {
    grid-template-columns: 1fr;
  }
  
  .items-grid {
    grid-template-columns: 1fr;
  }
  
  .dialog-footer {
    flex-direction: column;
    align-items: stretch;
  }

  .place-preview-grid {
    grid-template-columns: 1fr;
  }
}

@media (min-width: 1024px) {
  .rarity-break {
    display: block;
    grid-column: 1 / -1;
    height: 0;
  }
}
</style>