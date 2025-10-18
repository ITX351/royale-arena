<template>
  <el-card class="kill-record-display">
    <template #header>
      <div class="card-header">
        <h3>击杀记录</h3>
        <div class="header-controls">
          <el-select 
            v-if="isDirector" 
            v-model="filterForm.selectedKiller" 
            placeholder="筛选击杀者"
            clearable
            size="small"
            class="killer-filter"
          >
            <el-option label="无击杀者" value="__none__" />
            <el-option
              v-for="player in players"
              :key="player.id"
              :label="player.name"
              :value="player.id"
            />
          </el-select>
          <el-button-group>
            <el-button 
              :type="sortOrder === 'asc' ? 'primary' : 'default'" 
              @click="changeSortOrder('asc')"
              size="small"
            >
              时间正序
            </el-button>
            <el-button 
              :type="sortOrder === 'desc' ? 'primary' : 'default'" 
              @click="changeSortOrder('desc')"
              size="small"
            >
              时间倒序
            </el-button>
          </el-button-group>
        </div>
      </div>
    </template>
    
    <el-table 
      :data="filteredAndSortedRecords" 
      style="width: 100%" 
      size="small"
      max-height="400"
    >
      <el-table-column prop="kill_time" label="时间" width="160">
        <template #default="scope">
          {{ formatTime(scope.row.kill_time) }}
        </template>
      </el-table-column>
      <el-table-column prop="killer_name" label="击杀者" width="120">
        <template #default="scope">
          <span v-if="scope.row.killer_name">{{ scope.row.killer_name }}</span>
          <span v-else class="no-killer">无击杀者</span>
        </template>
      </el-table-column>
      <el-table-column prop="victim_name" label="被击杀者" width="120" />
      <el-table-column prop="cause" label="原因" width="120" />
      <el-table-column prop="weapon" label="武器" width="120">
        <template #default="scope">
          <span v-if="scope.row.weapon">{{ scope.row.weapon }}</span>
          <span v-else>-</span>
        </template>
      </el-table-column>
      <el-table-column prop="location" label="地点" width="120">
        <template #default="scope">
          <span v-if="scope.row.location">{{ scope.row.location }}</span>
          <span v-else>-</span>
        </template>
      </el-table-column>
    </el-table>
    
    <div v-if="filteredAndSortedRecords.length === 0" class="no-records">
      暂无击杀记录
    </div>
  </el-card>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import type { KillRecord } from '@/types/game'

interface Props {
  records: KillRecord[]
  players: Array<{ id: string; name: string }>
  isDirector?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  isDirector: false
})

// 响应式数据
const filterForm = ref({
  selectedKiller: ''
})

const sortOrder = ref<'asc' | 'desc'>('asc')

// 计算属性
const recordsWithPlayerNames = computed(() => {
  return props.records.map(record => {
    const killer = props.players.find(p => p.id === record.killer_id)
    const victim = props.players.find(p => p.id === record.victim_id)
    
    return {
      ...record,
      killer_name: killer ? killer.name : null,
      victim_name: victim ? victim.name : '未知'
    }
  })
})

const filteredRecords = computed(() => {
  if (!props.isDirector || !filterForm.value.selectedKiller) {
    return recordsWithPlayerNames.value
  }
  
  if (filterForm.value.selectedKiller === '__none__') {
    return recordsWithPlayerNames.value.filter(record => !record.killer_id)
  }
  
  return recordsWithPlayerNames.value.filter(
    record => record.killer_id === filterForm.value.selectedKiller
  )
})

const filteredAndSortedRecords = computed(() => {
  const sorted = [...filteredRecords.value]
  sorted.sort((a, b) => {
    const timeA = new Date(a.kill_time).getTime()
    const timeB = new Date(b.kill_time).getTime()
    
    if (sortOrder.value === 'asc') {
      return timeA - timeB
    } else {
      return timeB - timeA
    }
  })
  
  return sorted
})

// 方法
const formatTime = (timestamp: string) => {
  return new Date(timestamp).toLocaleString('zh-CN')
}

const changeSortOrder = (order: 'asc' | 'desc') => {
  sortOrder.value = order
}
</script>

<style scoped>
.kill-record-display {
  margin-top: 20px;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  flex-wrap: wrap;
  gap: 10px;
}

.header-controls {
  display: flex;
  gap: 10px;
  flex-wrap: wrap;
}

.killer-filter {
  width: 150px;
}

.no-killer {
  color: #909399;
  font-style: italic;
}

.no-records {
  text-align: center;
  padding: 20px;
  color: #909399;
}
</style>