<template>
  <el-dialog 
    v-model="dialogVisible" 
    :title="title" 
    :width="width"
    @close="handleClose"
  >
    <el-form :model="form" label-width="80px">
      <el-form-item :label="itemLabel">
        <el-select 
          v-model="form.selectedItem" 
          :placeholder="placeholder" 
          filterable
          clearable
          :style="{ width: '100%' }"
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
        <el-button @click="handleCancel">取消</el-button>
        <el-button 
          type="primary" 
          @click="handleConfirm"
          :disabled="!form.selectedItem"
        >
          确定
        </el-button>
      </span>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useGameStateStore } from '@/stores/gameState'
import { createItemParser, extractExistingItemsFromGameState } from '@/utils/itemParser'
import type { DirectorGameData } from '@/types/gameStateTypes'

// 定义组件属性
const props = defineProps<{
  modelValue: boolean
  title?: string
  itemLabel?: string
  placeholder?: string
  width?: string
  initialSelectedItem?: string
}>()

// 定义事件发射
const emit = defineEmits<{
  'update:modelValue': [value: boolean]
  confirm: [selectedItem: string]
  cancel: []
}>()

const store = useGameStateStore()

// 响应式数据
const form = ref({
  selectedItem: props.initialSelectedItem || ''
})

// 计算属性 - 复用AirdropPanel中的物品选项计算逻辑
const allItemOptions = computed(() => {
  const rulesJson = store.gameState?.global_state?.rules_config || {}
  
  try {
    // 获取现有物品
    const existingItems = extractExistingItemsFromGameState(store.gameData as DirectorGameData)
    
    // 创建物品解析器
    const parser = createItemParser(rulesJson, existingItems)
    
    // 获取可空投物品列表
    return parser.getAvailableAirdropItems()
  } catch (error) {
    console.error('解析物品列表失败:', error)
    return []
  }
})

// 计算属性 - 控制对话框显示
const dialogVisible = computed({
  get: () => props.modelValue,
  set: (value) => emit('update:modelValue', value)
})

// 监听器 - 当对话框打开时重置表单
watch(
  () => props.modelValue,
  (newVal) => {
    if (newVal) {
      form.value.selectedItem = props.initialSelectedItem || ''
    }
  }
)

// 方法实现
const handleConfirm = () => {
  emit('confirm', form.value.selectedItem)
  dialogVisible.value = false
}

const handleCancel = () => {
  emit('cancel')
  dialogVisible.value = false
}

const handleClose = () => {
  handleCancel()
}
</script>

<style scoped>
.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
}
</style>