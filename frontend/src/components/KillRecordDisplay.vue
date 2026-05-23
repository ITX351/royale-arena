<template>
  <div class="kill-record-display">
    <div v-if="props.showTitle" class="table-toolbar">
      <h3 class="toolbar-title">击杀记录</h3>
    </div>

    <el-table 
      ref="tableRef"
      :data="sortedRecords" 
      class="kill-record-table"
      size="small"
      max-height="400"
      empty-text="暂无击杀记录"
      :default-sort="{ prop: 'kill_time', order: 'ascending' }"
      @sort-change="handleSortChange"
    >
      <el-table-column
        prop="kill_time"
        label="时间"
        width="140"
        sortable="custom"
      >
        <template #default="scope">
          {{ formatTime(scope.row.kill_time) }}
        </template>
      </el-table-column>
      <el-table-column
        prop="killer_name"
        label="击杀者"
        width="110"
        sortable="custom"
      >
        <template #default="scope">
          <span v-if="scope.row.killer_name">{{ scope.row.killer_name }}</span>
          <span v-else class="no-killer">无击杀者</span>
        </template>
      </el-table-column>
      <el-table-column
        prop="victim_name"
        label="被击杀者"
        width="110"
        sortable="custom"
      />
      <el-table-column
        prop="cause"
        label="原因"
        width="110"
        sortable="custom"
      />
      <el-table-column
        prop="location"
        label="地点"
        width="110"
        sortable="custom"
      >
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
import type { TableInstance } from 'element-plus'
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

type TableSortOrder = 'ascending' | 'descending' | null
type SortOrder = Exclude<TableSortOrder, null>
type SortableProp = keyof KillRecord | 'killer_name' | 'victim_name'

const sortState = ref<{ prop: SortableProp; order: SortOrder }>({
  prop: 'kill_time',
  order: 'ascending'
})

const tableRef = ref<TableInstance>()

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

function compareByProp(
  a: Record<string, unknown>,
  b: Record<string, unknown>,
  prop: SortableProp,
  order: SortOrder
) {
  if (prop === 'kill_time') {
    const timeA = new Date(String(a[prop] ?? '')).getTime()
    const timeB = new Date(String(b[prop] ?? '')).getTime()
    const safeTimeA = Number.isNaN(timeA) ? 0 : timeA
    const safeTimeB = Number.isNaN(timeB) ? 0 : timeB
    return order === 'ascending' ? safeTimeA - safeTimeB : safeTimeB - safeTimeA
  }

  const toComparable = (value: unknown) => {
    if (value === null || value === undefined || value === '') {
      return ''
    }
    return String(value)
  }

  const valueA = toComparable(a[prop])
  const valueB = toComparable(b[prop])

  return order === 'ascending'
    ? valueA.localeCompare(valueB)
    : valueB.localeCompare(valueA)
}

const sortedRecords = computed(() => {
  const { prop, order } = sortState.value
  const sorted = [...recordsWithPlayerNames.value]
  sorted.sort((a, b) => compareByProp(a, b, prop, order))
  return sorted
})

// 方法
const formatTime = (timestamp: string) => {
  return new Date(timestamp).toLocaleString('zh-CN')
}

const handleSortChange = ({ prop, order }: { prop: string | null; order: TableSortOrder }) => {
  const resolvedProp = (prop as SortableProp) ?? sortState.value.prop ?? 'kill_time'
  const currentOrder = sortState.value.order
  const nextOrder: SortOrder = order === null
    ? (currentOrder === 'ascending' ? 'descending' : 'ascending')
    : order

  sortState.value = {
    prop: resolvedProp,
    order: nextOrder
  }

  tableRef.value?.sort(resolvedProp, nextOrder)
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

.toolbar-title {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
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