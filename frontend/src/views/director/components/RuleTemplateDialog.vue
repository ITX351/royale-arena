<template>
  <el-dialog
    v-model="visible"
    title="加载规则模版"
    width="720px"
    :close-on-click-modal="false"
    :destroy-on-close="false"
    @close="handleClose"
  >
    <div class="rule-template-dialog">
      <div class="tabs-with-action">
        <el-tabs v-model="activeTab">
        <el-tab-pane label="从规则模版加载" name="templates">
          <div class="tab-panel">
            <el-alert
              v-if="templateError"
              :title="templateError"
              type="error"
              show-icon
              :closable="false"
              class="alert-spacing"
            />

            <el-skeleton v-else-if="templateLoading" :rows="6" animated />

            <template v-else>
              <el-empty v-if="templates.length === 0" description="暂无规则模版" />
              <el-table
                v-else
                :data="templates"
                style="width: 100%"
                border
                height="360"
                highlight-current-row
                row-key="id"
                :current-row-key="selectedTemplateId"
                @row-click="selectTemplateRow"
              >
                <el-table-column label="选择" width="80">
                  <template #default="{ row }">
                    <el-radio v-model="selectedTemplateId" :label="row.id" />
                  </template>
                </el-table-column>
                <el-table-column prop="template_name" label="模版名称" min-width="180" />
                <el-table-column label="描述" min-width="260">
                  <template #default="{ row }">
                    {{ row.description || '暂无描述' }}
                  </template>
                </el-table-column>
                <el-table-column label="状态" width="100" align="center">
                  <template #default="{ row }">
                    <el-tag v-if="row.is_active" type="success">启用</el-tag>
                    <el-tag v-else type="info">停用</el-tag>
                  </template>
                </el-table-column>
              </el-table>
            </template>
          </div>
        </el-tab-pane>

        <el-tab-pane label="从已有游戏加载" name="games">
          <div class="tab-panel">
            <el-alert
              v-if="gamesError"
              :title="gamesError"
              type="error"
              show-icon
              :closable="false"
              class="alert-spacing"
            />

            <el-skeleton v-else-if="gamesLoading" :rows="6" animated />

            <template v-else>
              <el-empty v-if="games.length === 0" description="暂无游戏数据" />
              <el-table
                v-else
                :data="games"
                style="width: 100%"
                border
                height="360"
                highlight-current-row
                row-key="id"
                :current-row-key="selectedGameId"
                @row-click="selectGameRow"
              >
                <el-table-column label="选择" width="80">
                  <template #default="{ row }">
                    <el-radio v-model="selectedGameId" :label="row.id" />
                  </template>
                </el-table-column>
                <el-table-column prop="name" label="游戏名称" min-width="180" />
                <el-table-column label="描述" min-width="260">
                  <template #default="{ row }">
                    {{ row.description || '暂无描述' }}
                  </template>
                </el-table-column>
                <el-table-column label="状态" width="120" align="center">
                  <template #default="{ row }">
                    <el-tag :type="getStatusTagType(row.status)">
                      {{ getStatusLabel(row.status) }}
                    </el-tag>
                  </template>
                </el-table-column>
              </el-table>
            </template>
          </div>
        </el-tab-pane>
      </el-tabs>
        <el-button
          class="tabs-with-action__refresh"
          circle
          size="small"
          :icon="RefreshRight"
          @click="handleReload"
          :loading="currentLoading"
        />
      </div>
    </div>

    <template #footer>
      <span class="dialog-footer">
        <el-button @click="handleClose">取消</el-button>
        <el-button type="primary" @click="handleConfirm" :disabled="!selectedSource">
          确认加载
        </el-button>
      </span>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { ElMessage } from 'element-plus'
import { RefreshRight } from '@element-plus/icons-vue'
import { ruleTemplateService } from '@/services/ruleTemplateService'
import { gameService } from '@/services/gameService'
import { formatGameStatus, getStatusTagType } from '@/utils/gameUtils'
import type { RuleTemplate, RuleConfigSource } from '@/types/ruleTemplate'
import type { GameRulesConfigView } from '@/types/game'

const TAB_TEMPLATES = 'templates' as const
const TAB_GAMES = 'games' as const
type TabKey = typeof TAB_TEMPLATES | typeof TAB_GAMES

const props = defineProps<{ modelValue: boolean; currentGameId?: string }>()

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void
  (e: 'select', payload: RuleConfigSource): void
}>()

const visible = computed({
  get: () => props.modelValue,
  set: (value: boolean) => emit('update:modelValue', value)
})

const activeTab = ref<TabKey>(TAB_TEMPLATES)
const templates = ref<RuleTemplate[]>([])
const gamesRaw = ref<GameRulesConfigView[]>([])
const games = computed(() =>
  props.currentGameId
    ? gamesRaw.value.filter(item => item.id !== props.currentGameId)
    : gamesRaw.value
)
const templateLoading = ref(false)
const gamesLoading = ref(false)
const templateError = ref('')
const gamesError = ref('')
const selectedTemplateId = ref('')
const selectedGameId = ref('')

const selectedTemplate = computed(() =>
  templates.value.find(item => item.id === selectedTemplateId.value) ?? null
)

const selectedGame = computed(() =>
  games.value.find(item => item.id === selectedGameId.value) ?? null
)

const selectedSource = computed<RuleTemplate | GameRulesConfigView | null>(() =>
  activeTab.value === TAB_TEMPLATES ? selectedTemplate.value : selectedGame.value
)

const currentLoading = computed(() =>
  activeTab.value === TAB_TEMPLATES ? templateLoading.value : gamesLoading.value
)

const resetSelection = () => {
  selectedTemplateId.value = ''
  selectedGameId.value = ''
}

const transformToSource = (
  item: RuleTemplate | GameRulesConfigView
): RuleConfigSource => {
  if ('template_name' in item) {
    return {
      id: item.id,
      name: item.template_name,
      description: item.description,
      rules_config: item.rules_config,
      sourceType: 'template'
    }
  }

  return {
    id: item.id,
    name: item.name,
    description: item.description ?? null,
    rules_config: item.rules_config,
    sourceType: 'game'
  }
}

const getStatusLabel = (status: string) => formatGameStatus(status)

const loadTemplates = async () => {
  templateLoading.value = true
  templateError.value = ''

  try {
    const response = await ruleTemplateService.getTemplates()

    if (!response.success) {
      templates.value = []
      templateError.value = response.message || '加载规则模版失败'
      return
    }

    templates.value = response.data
  } catch (err: any) {
    templates.value = []
    templateError.value = err?.response?.data?.message || err?.message || '加载规则模版失败'
  } finally {
    templateLoading.value = false
  }
}

const loadGames = async () => {
  gamesLoading.value = true
  gamesError.value = ''

  try {
    const response = await gameService.getGamesRulesConfig()

    if (!response.success || !response.data) {
      gamesRaw.value = []
      gamesError.value = response?.message || '加载游戏规则配置失败'
      return
    }

    gamesRaw.value = response.data

    if (props.currentGameId && selectedGameId.value === props.currentGameId) {
      selectedGameId.value = ''
    }
  } catch (err: any) {
    gamesRaw.value = []
    gamesError.value = err?.response?.data?.message || err?.message || '加载游戏规则配置失败'
  } finally {
    gamesLoading.value = false
  }
}

const ensureGamesLoaded = () => {
  if (games.value.length === 0 && !gamesLoading.value) {
    loadGames()
  }
}

const selectTemplateRow = (row: RuleTemplate) => {
  selectedTemplateId.value = row.id
}

const selectGameRow = (row: GameRulesConfigView) => {
  selectedGameId.value = row.id
}

const handleReload = () => {
  if (activeTab.value === TAB_TEMPLATES) {
    loadTemplates()
  } else {
    loadGames()
  }
}

const handleConfirm = () => {
  const current = selectedSource.value

  if (!current) {
    const message =
      activeTab.value === TAB_TEMPLATES ? '请选择要加载的规则模版' : '请选择要加载的游戏'
    ElMessage.warning(message)
    return
  }

  emit('select', transformToSource(current))
  emit('update:modelValue', false)
  resetSelection()
}

const handleClose = () => {
  emit('update:modelValue', false)
  resetSelection()
  activeTab.value = TAB_TEMPLATES
}

watch(
  () => props.modelValue,
  (isVisible) => {
    if (isVisible) {
      activeTab.value = TAB_TEMPLATES
      resetSelection()
      loadTemplates()
    }
  }
)

watch(activeTab, (tab) => {
  if (tab === TAB_GAMES && visible.value) {
    ensureGamesLoaded()
  }
})

watch(
  () => props.currentGameId,
  (newId, oldId) => {
    if (newId === oldId) {
      return
    }

    if (newId && selectedGameId.value === newId) {
      selectedGameId.value = ''
    }

    if (visible.value && !gamesLoading.value) {
      loadGames()
    }
  }
)
</script>

<style scoped>
.rule-template-dialog {
  min-height: 360px;
}

.tabs-with-action {
  position: relative;
}

.tabs-with-action :deep(.el-tabs__header) {
  padding-right: 36px;
}

.tabs-with-action__refresh {
  position: absolute;
  top: 6px;
  right: 0;
}

.alert-spacing {
  margin-bottom: 16px;
}

.tab-panel {
  min-height: 360px;
}
</style>
