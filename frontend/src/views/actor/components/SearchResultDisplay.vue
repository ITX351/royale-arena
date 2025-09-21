<template>
  <el-card class="search-result-display">
    <template #header>
      <div class="card-header">
        <h3>搜索结果</h3>
        <el-tag 
          v-if="searchResult" 
          :type="searchResult.is_visible ? 'success' : 'warning'"
        >
          {{ searchResult.is_visible ? '可见' : '不可见' }}
        </el-tag>
      </div>
    </template>

    <div class="result-content">
      <!-- 无搜索结果提示 -->
      <el-empty 
        v-if="!player || !player.last_search_result" 
        description="暂无搜索结果" 
        :image-size="80"
      >
        <el-button 
          type="primary" 
          @click="performSearch"
          :disabled="!canSearch"
        >
          执行搜索
        </el-button>
      </el-empty>

      <!-- 搜索结果展示 -->
      <div v-else class="search-result">
        <div class="result-info">
          <div class="target-info">
            <el-avatar 
              :icon="searchResult && searchResult.target_type === 'player' ? 'User' : 'Box'" 
              :size="40"
            />
            <div class="target-details">
              <div class="target-name">
                {{ searchResult?.target_name || '' }}
                <el-tag :type="getTargetTypeTagType(searchResult?.target_type || '')">
                  {{ getTargetTypeLabel(searchResult?.target_type || '') }}
                </el-tag>
              </div>
              <div class="target-id" v-if="searchResult && searchResult.target_type === 'player'">
                ID: {{ searchResult?.target_id || '' }}
              </div>
            </div>
          </div>

          <div class="visibility-status">
            <el-tag :type="searchResult && searchResult.is_visible ? 'success' : 'warning'">
              {{ searchResult && searchResult.is_visible ? '可见' : '不可见' }}
            </el-tag>
          </div>
        </div>

        <div class="result-actions">
          <!-- 捡拾按钮 (物品可见时) -->
          <el-button 
            v-if="searchResult && searchResult.target_type === 'item' && searchResult.is_visible" 
            type="success" 
            @click="pickItem"
          >
            捡拾
          </el-button>

          <!-- 攻击按钮 (玩家可见时) -->
          <el-button 
            v-else-if="searchResult && searchResult.target_type === 'player' && searchResult.is_visible" 
            type="danger" 
            @click="attackPlayer"
          >
            攻击
          </el-button>

          <!-- 重新搜索按钮 -->
          <el-button 
            type="primary" 
            @click="performSearch"
            :disabled="!canSearch"
          >
            重新搜索
          </el-button>
        </div>
      </div>
    </div>
  </el-card>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { ElMessage } from 'element-plus'
import type { Player, SearchResult } from '@/types/gameStateTypes'

const props = defineProps<{
  player: Player | null
}>()

const emit = defineEmits<{
  (e: 'search'): void
  (e: 'pick'): void
  (e: 'attack', targetPlayerId: string): void
}>()

// 计算属性
const searchResult = computed((): SearchResult | null => {
  return props.player?.last_search_result || null
})

const canSearch = computed(() => {
  return props.player !== null && props.player.is_alive
})

// 方法
const getTargetTypeLabel = (targetType: string) => {
  switch (targetType) {
    case 'player': return '玩家'
    case 'item': return '物品'
    default: return '未知'
  }
}

const getTargetTypeTagType = (targetType: string) => {
  switch (targetType) {
    case 'player': return 'primary'
    case 'item': return 'success'
    default: return 'info'
  }
}

const performSearch = () => {
  if (!canSearch.value) {
    ElMessage.warning('当前无法执行搜索')
    return
  }
  
  // 发送搜索事件
  emit('search')
}

const pickItem = () => {
  if (!searchResult.value || searchResult.value.target_type !== 'item') {
    ElMessage.warning('当前搜索结果不是物品')
    return
  }
  
  if (!searchResult.value.is_visible) {
    ElMessage.warning('物品不可见，无法捡拾')
    return
  }
  
  // 发送捡拾事件
  emit('pick')
}

const attackPlayer = () => {
  if (!searchResult.value || searchResult.value.target_type !== 'player') {
    ElMessage.warning('当前搜索结果不是玩家')
    return
  }
  
  if (!searchResult.value.is_visible) {
    ElMessage.warning('玩家不可见，无法攻击')
    return
  }
  
  // 发送攻击事件
  emit('attack', searchResult.value.target_id)
}
</script>

<style scoped>
.search-result-display {
  background-color: #fff;
  border: 1px solid #e4e7ed;
}

.search-result-display :deep(.el-card__header) {
  background-color: #f5f7fa;
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

.result-content {
  display: flex;
  flex-direction: column;
  gap: 15px;
}

.search-result {
  display: flex;
  flex-direction: column;
  gap: 15px;
}

.result-info {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 10px;
  border: 1px solid #e4e7ed;
  border-radius: 4px;
  background-color: #fafafa;
}

.target-info {
  display: flex;
  align-items: center;
  gap: 10px;
}

.target-details {
  display: flex;
  flex-direction: column;
  gap: 3px;
}

.target-name {
  display: flex;
  align-items: center;
  gap: 5px;
  font-weight: 500;
  font-size: 16px;
  color: #333;
}

.target-id {
  font-size: 12px;
  color: #909399;
}

.result-actions {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
}

@media (max-width: 768px) {
  .result-info {
    flex-direction: column;
    align-items: flex-start;
    gap: 10px;
  }
  
  .result-actions {
    justify-content: flex-start;
  }
}
</style>