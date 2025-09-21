<template>
  <el-card class="airdrop-panel">
    <template #header>
      <div class="card-header">
        <h3>空投设置</h3>
      </div>
    </template>

    <div class="airdrop-content">
      <!-- 物品输入区域 -->
      <div class="item-input-section">
        <el-form :model="airdropForm" ref="airdropFormRef">
          <el-form-item label="空投物品">
            <div class="item-input-list">
              <div 
                v-for="(item, index) in airdropItems" 
                :key="index" 
                class="item-input-row"
              >
                <el-input 
                  v-model="item.name" 
                  placeholder="输入物品名称"
                  clearable
                />
                <el-button 
                  type="danger" 
                  circle 
                  @click="removeItem(index)"
                >
                  <el-icon><Delete /></el-icon>
                </el-button>
              </div>
              <el-button 
                type="primary" 
                @click="addItem"
                class="add-item-btn"
              >
                <el-icon><Plus /></el-icon>
                添加物品
              </el-button>
            </div>
          </el-form-item>
        </el-form>
      </div>

      <!-- 随机生成按钮 -->
      <div class="random-generate-section">
        <el-button 
          type="success" 
          @click="generateRandomItems"
          :loading="generating"
        >
          随机生成空投
        </el-button>
      </div>

      <!-- 预览区域 -->
      <div class="preview-section" v-if="generatedItems.length > 0">
        <h4>预览</h4>
        <el-table :data="generatedItems" style="width: 100%">
          <el-table-column prop="name" label="物品名称" />
          <el-table-column label="操作">
            <template #default="{ row }">
              <el-button 
                type="danger" 
                size="small" 
                @click="removeGeneratedItem(row)"
              >
                移除
              </el-button>
            </template>
          </el-table-column>
        </el-table>
      </div>

      <!-- 操作按钮 -->
      <div class="action-buttons" v-if="generatedItems.length > 0">
        <el-button 
          type="primary" 
          @click="acceptAirdrop"
          :loading="accepting"
        >
          接受空投
        </el-button>
        <el-button @click="rejectAirdrop">拒绝</el-button>
      </div>
    </div>
  </el-card>
</template>

<script setup lang="ts">
import { ref, reactive } from 'vue'
import { ElMessage } from 'element-plus'
import type { AirdropItem } from '@/types/gameStateTypes'

// 定义组件属性
const props = defineProps<{
  gameId: string
}>()

// 定义事件发射
const emit = defineEmits<{
  (e: 'airdrop-accepted', items: AirdropItem[], place: string): void
}>()

// 响应式状态
const airdropFormRef = ref()
const airdropForm = reactive({
  items: [] as AirdropItem[]
})

const airdropItems = ref<AirdropItem[]>([{ id: '1', name: '' }])
const generatedItems = ref<AirdropItem[]>([])
const generating = ref(false)
const accepting = ref(false)

// 方法实现
const addItem = () => {
  const newId = Date.now().toString()
  airdropItems.value.push({ id: newId, name: '' })
}

const removeItem = (index: number) => {
  if (airdropItems.value.length > 1) {
    airdropItems.value.splice(index, 1)
  } else {
    airdropItems.value[0].name = ''
  }
}

const generateRandomItems = () => {
  generating.value = true
  
  // 模拟随机生成过程
  setTimeout(() => {
    const randomItems = [
      { id: '1', name: '医疗包' },
      { id: '2', name: '能量饮料' },
      { id: '3', name: '手枪' },
      { id: '4', name: '子弹' },
      { id: '5', name: '防护服' }
    ]
    
    // 随机选择1-3个物品
    const itemCount = Math.floor(Math.random() * 3) + 1
    generatedItems.value = []
    
    for (let i = 0; i < itemCount; i++) {
      const randomIndex = Math.floor(Math.random() * randomItems.length)
      const item = randomItems[randomIndex]
      generatedItems.value.push({ ...item, id: `${Date.now()}-${i}` })
    }
    
    generating.value = false
    ElMessage.success('随机空投生成成功')
  }, 500)
}

const removeGeneratedItem = (item: AirdropItem) => {
  const index = generatedItems.value.findIndex(i => i.id === item.id)
  if (index !== -1) {
    generatedItems.value.splice(index, 1)
  }
}

const acceptAirdrop = () => {
  if (generatedItems.value.length === 0) {
    ElMessage.warning('请先生成空投物品')
    return
  }
  
  // 这里应该弹出地点选择对话框，为了简化直接使用默认地点
  const placeName = '中心广场' // 实际应用中应该让用户选择地点
  
  accepting.value = true
  
  // 模拟发送空投请求
  setTimeout(() => {
    emit('airdrop-accepted', generatedItems.value, placeName)
    ElMessage.success('空投已发送')
    generatedItems.value = []
    accepting.value = false
  }, 500)
}

const rejectAirdrop = () => {
  generatedItems.value = []
  ElMessage.info('已取消空投')
}
</script>

<style scoped>
.airdrop-panel {
  margin-bottom: 20px;
}

.card-header h3 {
  margin: 0;
  color: #303133;
}

.item-input-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.item-input-row {
  display: flex;
  align-items: center;
  gap: 10px;
}

.add-item-btn {
  align-self: flex-start;
}

.random-generate-section {
  margin: 20px 0;
  text-align: center;
}

.preview-section {
  margin: 20px 0;
}

.preview-section h4 {
  margin-bottom: 15px;
  color: #606266;
}

.action-buttons {
  display: flex;
  justify-content: center;
  gap: 20px;
  margin-top: 20px;
}

@media (max-width: 768px) {
  .item-input-row {
    flex-direction: column;
    align-items: stretch;
  }
  
  .action-buttons {
    flex-direction: column;
    align-items: center;
  }
}
</style>