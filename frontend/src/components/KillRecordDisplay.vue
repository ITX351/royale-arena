<template>
  <div class="kill-record-display">
    <div
      v-if="props.showTitle || props.isDirector"
      class="table-toolbar"
      :class="{ 'table-toolbar--no-title': !props.showTitle }"
    >
      <h3 v-if="props.showTitle" class="toolbar-title">击杀记录</h3>
      <div class="toolbar-controls">
        <el-select 
          v-if="props.isDirector" 
          v-model="filterForm.selectedKiller" 
          placeholder="筛选击杀者"
          clearable
          filterable
          size="small"
          class="killer-filter"
        >
          <el-option label="无击杀者" value="__none__" />
          <el-option
            v-for="player in sortedPlayers"
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

    <el-table 
      :data="filteredAndSortedRecords" 
      class="kill-record-table"
      size="small"
      max-height="400"
      empty-text="暂无击杀记录"
    >
      <el-table-column prop="kill_time" label="时间" width="140">
        <template #default="scope">
          {{ formatTime(scope.row.kill_time) }}
        </template>
      </el-table-column>
      <el-table-column prop="killer_name" label="击杀者" width="110">
        <template #default="scope">
          <span v-if="scope.row.killer_name">{{ scope.row.killer_name }}</span>
          <span v-else class="no-killer">无击杀者</span>
        </template>
      </el-table-column>
      <el-table-column prop="victim_name" label="被击杀者" width="110" />
      <el-table-column prop="cause" label="原因" width="110" />
      <el-table-column prop="location" label="地点" width="110">
        <template #default="scope">
          <span v-if="scope.row.location">{{ scope.row.location }}</span>
          <span v-else>-</span>
        </template>
      </el-table-column>
    </el-table>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import type { KillRecord } from '@/types/game'

interface Props {
  records: KillRecord[]
  players: Array<{ id: string; name: string }>
  isDirector?: boolean
  showTitle?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  isDirector: false,
  showTitle: true
})

// 响应式数据
const filterForm = ref({
  selectedKiller: ''
})

const sortOrder = ref<'asc' | 'desc'>('asc')

// Sort players so the select remains easy to scan when many entries exist.
const sortedPlayers = computed(() => {
  return [...props.players].sort((a, b) => a.name.localeCompare(b.name))
})

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
  margin: 12px auto 0;
  max-width: 640px;
  width: 100%;
}

.table-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  flex-wrap: wrap;
  margin-bottom: 12px;
}

.table-toolbar--no-title {
  justify-content: flex-end;
}

.toolbar-title {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
}

.toolbar-controls {
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

.kill-record-table {
  width: 100%;
  max-width: 640px;
  margin: 0 auto;
}

.kill-record-display :deep(.el-table__header-wrapper table),
.kill-record-display :deep(.el-table__body-wrapper table) {
  margin: 0 auto;
  display: inline-table;
}

@media (max-width: 600px) {
  .table-toolbar {
    justify-content: center;
  }
}
</style>