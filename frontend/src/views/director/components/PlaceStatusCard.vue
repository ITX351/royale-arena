<template>
  <el-card class="place-status-card">
    <template #header>
      <div class="card-header">
        <h3>地点状态管理</h3>
      </div>
    </template>
    <div class="place-status-content">
      <el-table :data="placeList" style="width: 100%" size="small" max-height="200">
        <el-table-column prop="name" label="地点名称" />
        <el-table-column label="状态">
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
      </el-table>
    </div>
  </el-card>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useGameStateStore } from '@/stores/gameState'
import type { Place } from '@/types/directorGameState'

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

// 地点状态调整方法
const handlePlaceStatusChange = (placeName: string, isDestroyed: boolean | string | number) => {
  // 确保isDestroyed是布尔值
  const isDestroyedBool = Boolean(isDestroyed)
  // 调用store中的方法调整地点状态
  store.togglePlaceStatus(placeName, isDestroyedBool)
  // 发送事件通知父组件
  emit('place-status-change', placeName, isDestroyedBool)
}
</script>

<style scoped>
.place-status-card {
  margin-bottom: 20px;
}

.card-header h3 {
  margin: 0;
  color: #303133;
}

.place-status-content {
  padding: 10px 0;
}
</style>