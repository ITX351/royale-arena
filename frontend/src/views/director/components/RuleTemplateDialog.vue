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
      <div class="dialog-toolbar">
        <el-button @click="loadTemplates" :loading="loading">
          重新加载
        </el-button>
      </div>

      <el-alert
        v-if="error"
        :title="error"
        type="error"
        show-icon
        :closable="false"
        class="alert-spacing"
      />

      <el-skeleton v-else-if="loading" :rows="6" animated />

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
          @row-click="selectRow"
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

    <template #footer>
      <span class="dialog-footer">
        <el-button @click="handleClose">取消</el-button>
        <el-button type="primary" @click="handleConfirm" :disabled="!selectedTemplate">
          确认加载
        </el-button>
      </span>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { ElMessage } from 'element-plus'
import { ruleTemplateService } from '@/services/ruleTemplateService'
import type { RuleTemplate } from '@/types/ruleTemplate'

const props = defineProps<{
  modelValue: boolean
}>()

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void
  (e: 'select', template: RuleTemplate): void
}>()

const visible = computed({
  get: () => props.modelValue,
  set: (value: boolean) => emit('update:modelValue', value)
})

const templates = ref<RuleTemplate[]>([])
const loading = ref(false)
const error = ref('')
const selectedTemplateId = ref('')

const selectedTemplate = computed(() =>
  templates.value.find(item => item.id === selectedTemplateId.value) ?? null
)

const loadTemplates = async () => {
  loading.value = true
  error.value = ''

  try {
    const response = await ruleTemplateService.getTemplates()

    if (!response.success) {
      templates.value = []
      error.value = response.message || '加载规则模版失败'
      return
    }

    templates.value = response.data
  } catch (err: any) {
    templates.value = []
    error.value = err?.response?.data?.message || err?.message || '加载规则模版失败'
  } finally {
    loading.value = false
  }
}

const selectRow = (row: RuleTemplate) => {
  selectedTemplateId.value = row.id
}

const handleConfirm = () => {
  if (!selectedTemplate.value) {
    ElMessage.warning('请选择要加载的规则模版')
    return
  }

  emit('select', selectedTemplate.value)
  emit('update:modelValue', false)
  selectedTemplateId.value = ''
}

const handleClose = () => {
  emit('update:modelValue', false)
  selectedTemplateId.value = ''
}

watch(
  () => props.modelValue,
  (isVisible) => {
    if (isVisible) {
      selectedTemplateId.value = ''
      loadTemplates()
    }
  }
)
</script>

<style scoped>
.rule-template-dialog {
  min-height: 360px;
}

.dialog-toolbar {
  display: flex;
  justify-content: flex-end;
  margin-bottom: 12px;
}

.alert-spacing {
  margin-bottom: 16px;
}
</style>
