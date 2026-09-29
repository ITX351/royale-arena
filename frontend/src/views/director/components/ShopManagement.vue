<template>
  <el-card class="shop-management-card">
    <template #header>
      <div class="card-header">
        <h4>商店管理</h4>
        <el-button type="primary" size="small" :icon="Plus" @click="openListDialog">
          上架物品
        </el-button>
      </div>
    </template>

    <el-table
      v-if="shopListings.length > 0"
      :data="shopListings"
      size="small"
      stripe
      border
    >
      <el-table-column label="物品名称">
        <template #default="{ row }">
          <span v-if="row.rarity" :class="['rarity-dot', row.rarity]"></span>
          {{ row.item_name }}
        </template>
      </el-table-column>
      <el-table-column prop="price" label="单价（币）" width="120" />
      <el-table-column prop="quantity" label="库存" width="80" />
      <el-table-column label="操作" width="80">
        <template #default="{ row }">
          <el-button type="danger" size="small" @click="handleDelist(row.id)">
            下架
          </el-button>
        </template>
      </el-table-column>
    </el-table>
    <el-empty v-else description="暂无上架物品" :image-size="60" />

    <!-- 上架对话框 -->
    <el-dialog
      v-model="dialogVisible"
      title="上架物品"
      width="420px"
      :close-on-click-modal="false"
    >
      <el-form label-width="80px">
        <el-form-item label="上架类型">
          <el-select :model-value="listMode" style="width: 100%" @update:model-value="onModeChange">
            <el-option label="具体物品（消耗品等）" value="exact" />
            <el-option label="武器（按稀有度随机）" value="weapon" />
            <el-option label="防具（按稀有度随机）" value="armor" />
          </el-select>
        </el-form-item>
        <el-form-item v-if="listMode === 'exact'" label="物品">
          <el-select
            v-model="selectedItem"
            placeholder="选择物品"
            filterable
            style="width: 100%"
          >
            <el-option-group
              v-for="group in itemGroups"
              :key="group.label"
              :label="group.label"
            >
              <el-option
                v-for="name in group.items"
                :key="name"
                :label="name"
                :value="name"
              />
            </el-option-group>
          </el-select>
        </el-form-item>
        <el-form-item v-else label="稀有度">
          <el-select v-model="selectedRarity" placeholder="选择稀有度" style="width: 100%">
            <el-option
              v-for="o in rarityOptions"
              :key="o.rarity"
              :label="`${rarityLabel(o.rarity)}（可抽 ${o.size} 件）`"
              :value="o.rarity"
            />
          </el-select>
        </el-form-item>
        <el-form-item label="单价">
          <el-input-number
            v-model="price"
            :min="1"
            :max="9999"
            style="width: 100%"
          />
        </el-form-item>
        <el-form-item label="数量">
          <el-input-number
            v-model="quantity"
            :min="1"
            :max="maxQuantity"
            style="width: 100%"
          />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="dialogVisible = false">取消</el-button>
        <el-button
          type="primary"
          :disabled="!canSubmit"
          @click="handleListItem"
        >
          上架
        </el-button>
      </template>
    </el-dialog>
  </el-card>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { Plus } from '@element-plus/icons-vue'
import { useGameStateStore } from '@/stores/gameState'
import { createItemParser, type ParsedItemInfo } from '@/utils/itemParser'

const store = useGameStateStore()

const dialogVisible = ref(false)
const selectedItem = ref('')
const price = ref(1)
const quantity = ref(1)

const shopListings = computed(() => store.shopListings)

const rulesJson = computed(() => store.globalState?.rules_config)

const parsedItems = computed<ParsedItemInfo | null>(() => {
  if (!rulesJson.value) return null
  try {
    const parser = createItemParser(rulesJson.value, [])
    return parser.parseAllItems()
  } catch (error) {
    console.error('解析物品失败:', error)
    return null
  }
})

interface ItemGroup {
  label: string
  items: string[]
}

const itemGroups = computed<ItemGroup[]>(() => {
  if (!parsedItems.value) return []
  const groups: ItemGroup[] = []
  const p = parsedItems.value

  if (p.utilities.length > 0) {
    groups.push({ label: '功能道具', items: p.utilities })
  }
  if (p.consumables.length > 0) {
    groups.push({ label: '消耗品', items: p.consumables })
  }
  if (p.currencies.length > 0) {
    groups.push({ label: '货币', items: p.currencies })
  }
  if (p.permanentBuffs.length > 0) {
    groups.push({ label: '永久增益', items: p.permanentBuffs })
  }
  if (p.upgraders.length > 0) {
    groups.push({ label: '升级器', items: p.upgraders })
  }

  return groups
})

type ListMode = 'weapon' | 'armor' | 'exact'

const listMode = ref<ListMode>('exact')
const selectedRarity = ref('')

const ALL_RARITIES = ['common', 'rare', 'epic', 'legendary'] as const
const rarityLabel = (r: string) =>
  ({ common: '绿', rare: '蓝', epic: '紫', legendary: '橙' })[r] || r

const rarityPool = computed(() =>
  listMode.value === 'weapon'
    ? parsedItems.value?.rarityItems.weapons
    : parsedItems.value?.rarityItems.armors
)

// 只列未上架且道具库非空的稀有度
const rarityOptions = computed(() =>
  ALL_RARITIES.map((r) => ({ rarity: r, size: (rarityPool.value?.[r] || []).length })).filter(
    (o) =>
      o.size > 0 &&
      !shopListings.value.some((l) => l.item_kind === listMode.value && l.rarity === o.rarity)
  )
)

const maxQuantity = computed(() => {
  if (listMode.value === 'exact') return 999
  const size = (rarityPool.value?.[selectedRarity.value] || []).length
  return size > 0 ? size : 1
})

const switchMode = (mode: ListMode) => {
  listMode.value = mode
  selectedRarity.value = ''
  selectedItem.value = ''
  quantity.value = 1
}

const onModeChange = (value: string | number | boolean | object | undefined) => {
  switchMode(value as ListMode)
}

const canSubmit = computed(() => {
  if (price.value < 1 || quantity.value < 1 || quantity.value > maxQuantity.value) return false
  return listMode.value === 'exact' ? !!selectedItem.value : !!selectedRarity.value
})

const openListDialog = () => {
  switchMode('exact')
  price.value = 1
  dialogVisible.value = true
}

const handleListItem = () => {
  if (price.value < 1 || quantity.value < 1) return
  if (listMode.value === 'exact') {
    if (!selectedItem.value) return
    store.shopListItem(selectedItem.value, price.value, quantity.value)
  } else {
    if (!selectedRarity.value) return
    store.shopListRarity(listMode.value, selectedRarity.value, price.value, quantity.value)
  }
  dialogVisible.value = false
}

const handleDelist = (listingId: string) => {
  store.shopDelistItem(listingId)
}
</script>

<style scoped>
.shop-management-card {
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

.rarity-dot {
  display: inline-block;
  width: 8px;
  height: 8px;
  border-radius: 50%;
  margin-right: 6px;
  vertical-align: middle;
}
.rarity-dot.common {
  background-color: #67c23a;
}
.rarity-dot.rare {
  background-color: #409eff;
}
.rarity-dot.epic {
  background-color: #9b59b6;
}
.rarity-dot.legendary {
  background-color: #e6a23c;
}
</style>
