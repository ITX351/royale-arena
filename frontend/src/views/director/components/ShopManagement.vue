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
      <el-table-column prop="item_name" label="物品名称" />
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
        <el-form-item label="物品">
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
            :max="999"
            style="width: 100%"
          />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="dialogVisible = false">取消</el-button>
        <el-button
          type="primary"
          :disabled="!selectedItem || price < 1 || quantity < 1"
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

  if (Object.values(p.rarityItems.weapons).flat().length > 0) {
    groups.push({ label: '武器', items: Object.values(p.rarityItems.weapons).flat() })
  }
  if (Object.values(p.rarityItems.armors).flat().length > 0) {
    groups.push({ label: '防具', items: Object.values(p.rarityItems.armors).flat() })
  }
  if (p.utilities.length > 0) {
    groups.push({ label: '功能道具', items: p.utilities })
  }
  if (p.consumables.length > 0) {
    groups.push({ label: '消耗品', items: p.consumables })
  }
  if (p.currencies.length > 0) {
    groups.push({ label: '货币', items: p.currencies })
  }
  if (p.upgraders.length > 0) {
    groups.push({ label: '升级器', items: p.upgraders })
  }

  return groups
})

const openListDialog = () => {
  selectedItem.value = ''
  price.value = 1
  quantity.value = 1
  dialogVisible.value = true
}

const handleListItem = () => {
  if (!selectedItem.value || price.value < 1 || quantity.value < 1) return
  store.shopListItem(selectedItem.value, price.value, quantity.value)
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
</style>
