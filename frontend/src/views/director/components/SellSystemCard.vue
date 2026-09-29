<template>
  <el-card class="sell-system-card">
    <template #header>
      <div class="card-header">
        <h4>售出系统</h4>
        <el-button
          type="primary"
          size="small"
          :icon="Plus"
          :disabled="availableRarities.length === 0"
          @click="openAddDialog"
        >
          添加稀有度价格
        </el-button>
      </div>
    </template>

    <el-table v-if="sellPrices.length > 0" :data="sellPrices" size="small" stripe border>
      <el-table-column label="稀有度" width="120">
        <template #default="{ row }">
          <span :class="['rarity-tag', row.rarity]">{{ rarityLabel(row.rarity) }}</span>
        </template>
      </el-table-column>
      <el-table-column label="售出价（币）" width="140">
        <template #default="{ row }">{{ formatPrice(row.price) }}</template>
      </el-table-column>
      <el-table-column label="操作" width="160">
        <template #default="{ row }">
          <el-button type="primary" size="small" @click="openEditDialog(row)">改价</el-button>
          <el-button type="danger" size="small" @click="handleRemove(row.rarity)">删除</el-button>
        </template>
      </el-table-column>
    </el-table>
    <el-empty v-else description="暂未配置售出价格（玩家将无法售出道具）" :image-size="60" />

    <el-dialog
      v-model="dialogVisible"
      :title="editingRarity ? '修改售出价格' : '添加售出价格'"
      width="420px"
      :close-on-click-modal="false"
    >
      <el-form label-width="80px">
        <el-form-item label="稀有度">
          <el-select
            v-model="selectedRarity"
            :disabled="!!editingRarity"
            placeholder="选择稀有度"
            style="width: 100%"
          >
            <el-option
              v-for="r in editingRarity ? [editingRarity] : availableRarities"
              :key="r"
              :label="rarityLabel(r)"
              :value="r"
            />
          </el-select>
        </el-form-item>
        <el-form-item label="价格（币）">
          <el-input-number
            v-model="price"
            :min="0.5"
            :max="9999"
            :step="0.5"
            style="width: 100%"
          />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="dialogVisible = false">取消</el-button>
        <el-button type="primary" :disabled="!selectedRarity || price < 0.5" @click="handleSave">
          保存
        </el-button>
      </template>
    </el-dialog>
  </el-card>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { Plus } from '@element-plus/icons-vue'
import { useGameStateStore } from '@/stores/gameState'
import type { SellPriceEntry } from '@/types/gameStateTypes'

const store = useGameStateStore()

const ALL_RARITIES = ['common', 'rare', 'epic', 'legendary'] as const

const sellPrices = computed(() => store.sellPrices)

const availableRarities = computed(() =>
  ALL_RARITIES.filter((r) => !sellPrices.value.some((e) => e.rarity === r))
)

const dialogVisible = ref(false)
const editingRarity = ref<string | null>(null)
const selectedRarity = ref('')
const price = ref(0.5)

const rarityLabel = (rarity: string) =>
  ({ common: '绿', rare: '蓝', epic: '紫', legendary: '橙' })[rarity] || rarity

const formatPrice = (p: number) => String(p)

const openAddDialog = () => {
  editingRarity.value = null
  selectedRarity.value = ''
  price.value = 0.5
  dialogVisible.value = true
}

const openEditDialog = (row: SellPriceEntry) => {
  editingRarity.value = row.rarity
  selectedRarity.value = row.rarity
  price.value = row.price
  dialogVisible.value = true
}

const handleSave = () => {
  if (!selectedRarity.value || price.value < 0.5) return
  store.sellSetPrice(selectedRarity.value, price.value)
  dialogVisible.value = false
}

const handleRemove = (rarity: string) => {
  store.sellRemovePrice(rarity)
}
</script>

<style scoped>
.sell-system-card {
  width: 100%;
}
.card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}
.card-header h4 {
  margin: 0;
  color: #606266;
  font-size: 16px;
  font-weight: 600;
}
.rarity-tag {
  display: inline-block;
  padding: 2px 8px;
  border-radius: 4px;
  color: #fff;
  font-size: 12px;
}
.rarity-tag.common { background-color: #67c23a; }
.rarity-tag.rare { background-color: #409eff; }
.rarity-tag.epic { background-color: #9b59b6; }
.rarity-tag.legendary { background-color: #e6a23c; }
</style>
