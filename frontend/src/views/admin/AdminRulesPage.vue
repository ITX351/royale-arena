<template>
  <div class="admin-rules-page">
    <div class="page-header">
      <div class="header-left">
        <h3 class="page-title">规则模版管理</h3>
        <p class="page-subtitle">维护游戏规则模版，支持快速创建、编辑与删除操作</p>
      </div>
      <div class="header-right">
        <el-button :icon="Refresh" @click="loadTemplates" :loading="loading">
          刷新
        </el-button>
        <el-button type="primary" :icon="Plus" @click="openCreateDialog">
          创建模版
        </el-button>
      </div>
    </div>

    <div class="filter-bar">
      <el-input
        v-model="searchKeyword"
        placeholder="搜索模版名称或描述..."
        :prefix-icon="Search"
        clearable
        class="filter-input"
      />
      <el-select v-model="statusFilter" class="filter-select">
        <el-option label="全部状态" value="all" />
        <el-option label="已启用" value="active" />
        <el-option label="已停用" value="inactive" />
      </el-select>
    </div>

    <el-card class="table-card">
      <el-table
        :data="filteredTemplates"
        v-loading="loading"
        empty-text="暂无规则模版数据"
        border
      >
        <el-table-column prop="template_name" label="模版名称">
          <template #default="{ row }">
            <div class="template-name">
              <span class="name-text">{{ row.template_name }}</span>
              <el-tag
                size="small"
                :type="row.is_active ? 'success' : 'info'"
                effect="dark"
              >
                {{ row.is_active ? '启用' : '停用' }}
              </el-tag>
            </div>
          </template>
        </el-table-column>

        <el-table-column prop="description" label="描述" show-overflow-tooltip />

        <el-table-column label="启用状态" width="140" align="center">
          <template #default="{ row }">
            <el-switch
              :model-value="row.is_active"
              inline-prompt
              active-text="启用"
              inactive-text="停用"
              :loading="togglingId === row.id"
              @change="(value: boolean) => handleToggleActive(row, value)"
            />
          </template>
        </el-table-column>

        <el-table-column prop="updated_at" label="最后更新" width="180">
          <template #default="{ row }">
            {{ formatDateTime(row.updated_at) }}
          </template>
        </el-table-column>

        <el-table-column label="操作" width="280" fixed="right">
          <template #default="{ row }">
            <div class="action-buttons">
              <el-button type="primary" size="small" :icon="View" text @click="previewConfig(row)">
                查看配置
              </el-button>
              <el-button type="primary" size="small" :icon="Edit" text @click="openEditDialog(row)">
                编辑
              </el-button>
              <el-button type="danger" size="small" :icon="Delete" text @click="confirmDelete(row)">
                删除
              </el-button>
            </div>
          </template>
        </el-table-column>
      </el-table>
    </el-card>

    <el-dialog
      v-model="showTemplateDialog"
      :title="dialogTitle"
      width="640px"
      :close-on-click-modal="false"
    >
      <el-form
        ref="templateFormRef"
        :model="templateForm"
        :rules="templateFormRules"
        label-width="100px"
      >
        <el-form-item label="模版名称" prop="template_name">
          <el-input v-model="templateForm.template_name" placeholder="请输入模版名称" />
        </el-form-item>

        <el-form-item label="模版描述" prop="description">
          <el-input
            v-model="templateForm.description"
            type="textarea"
            :rows="3"
            placeholder="请输入模版描述（可选）"
          />
        </el-form-item>

        <el-form-item label="启用状态" prop="is_active">
          <el-switch
            v-model="templateForm.is_active"
            inline-prompt
            active-text="启用"
            inactive-text="停用"
          />
        </el-form-item>

        <el-form-item label="规则配置" prop="rules_config_text">
          <el-input
            v-model="templateForm.rules_config_text"
            type="textarea"
            :rows="12"
            placeholder="请输入合法的 JSON 对象"
            resize="none"
          />
        </el-form-item>
      </el-form>

      <template #footer>
        <el-button @click="closeTemplateDialog">取消</el-button>
        <el-button type="primary" :loading="saving" @click="submitTemplateForm">
          {{ dialogMode === 'create' ? '创建' : '保存' }}
        </el-button>
      </template>
    </el-dialog>

    <el-dialog
      v-model="previewVisible"
      title="规则配置"
      width="640px"
      destroy-on-close
    >
      <div class="json-preview" v-if="previewJson">
        <pre>{{ previewJson }}</pre>
      </div>
      <template #footer>
        <el-button @click="previewVisible = false">关闭</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted } from 'vue'
import { ElMessage, ElMessageBox, type FormInstance, type FormRules } from 'element-plus'
import { Plus, Refresh, Edit, Delete, View, Search } from '@element-plus/icons-vue'
import { adminService } from '@/services/adminService'
import type {
  RuleTemplate,
  CreateRuleTemplateRequest,
  UpdateRuleTemplateRequest
} from '@/types/admin'
import { formatDateTime } from '@/utils/gameFilter'

type TemplateDialogMode = 'create' | 'edit'

const loading = ref(false)
const saving = ref(false)
const templates = ref<RuleTemplate[]>([])
const searchKeyword = ref('')
const statusFilter = ref<'all' | 'active' | 'inactive'>('all')
const showTemplateDialog = ref(false)
const dialogMode = ref<TemplateDialogMode>('create')
const editingTemplateId = ref<string | null>(null)
const previewVisible = ref(false)
const previewTemplate = ref<RuleTemplate | null>(null)
const togglingId = ref<string | null>(null)

const templateFormRef = ref<FormInstance>()
const templateForm = reactive({
  template_name: '',
  description: '',
  is_active: true,
  rules_config_text: '{}'
})

const templateFormRules: FormRules = {
  template_name: [
    { required: true, message: '请输入模版名称', trigger: 'blur' },
    { min: 1, max: 100, message: '模版名称长度在 1 到 100 个字符', trigger: 'blur' }
  ],
  rules_config_text: [
    { required: true, message: '请填写规则配置', trigger: 'blur' },
    {
      validator: (_rule, value: string, callback) => {
        try {
          const parsed = JSON.parse(value)
          if (!parsed || typeof parsed !== 'object' || Array.isArray(parsed)) {
            callback(new Error('规则配置必须是 JSON 对象'))
            return
          }
          callback()
        } catch {
          callback(new Error('规则配置需为合法的 JSON 格式'))
        }
      },
      trigger: ['blur', 'change']
    }
  ]
}

const dialogTitle = computed(() =>
  dialogMode.value === 'create' ? '创建规则模版' : '编辑规则模版'
)

const previewJson = computed(() =>
  previewTemplate.value ? JSON.stringify(previewTemplate.value.rules_config, null, 2) : ''
)

const filteredTemplates = computed(() => {
  const keyword = searchKeyword.value.trim().toLowerCase()

  return templates.value
    .filter(template => {
      if (statusFilter.value === 'active') {
        return template.is_active
      }
      if (statusFilter.value === 'inactive') {
        return !template.is_active
      }
      return true
    })
    .filter(template => {
      if (!keyword) {
        return true
      }
      const nameMatched = template.template_name.toLowerCase().includes(keyword)
      const descriptionMatched = template.description?.toLowerCase().includes(keyword)
      return nameMatched || !!descriptionMatched
    })
})

const resetTemplateForm = () => {
  templateForm.template_name = ''
  templateForm.description = ''
  templateForm.is_active = true
  templateForm.rules_config_text = '{}'
  templateFormRef.value?.clearValidate()
}

const loadTemplates = async () => {
  loading.value = true
  try {
    const response = await adminService.getRuleTemplates()
    if (response.success && response.data) {
      templates.value = response.data
    } else {
      throw new Error(response.error?.message || '获取规则模版失败')
    }
  } catch (error) {
    console.error('加载规则模版失败:', error)
    ElMessage.error('加载规则模版失败')
  } finally {
    loading.value = false
  }
}

const openCreateDialog = () => {
  dialogMode.value = 'create'
  editingTemplateId.value = null
  resetTemplateForm()
  templateForm.rules_config_text = '{\n  "game_flow": {}\n}'
  showTemplateDialog.value = true
}

const openEditDialog = (template: RuleTemplate) => {
  dialogMode.value = 'edit'
  editingTemplateId.value = template.id
  templateForm.template_name = template.template_name
  templateForm.description = template.description || ''
  templateForm.is_active = template.is_active
  templateForm.rules_config_text = JSON.stringify(template.rules_config, null, 2)
  templateFormRef.value?.clearValidate()
  showTemplateDialog.value = true
}

const closeTemplateDialog = () => {
  showTemplateDialog.value = false
}

const submitTemplateForm = () => {
  if (!templateFormRef.value) {
    return
  }

  templateFormRef.value.validate(async valid => {
    if (!valid) {
      return
    }

    let parsedConfig: Record<string, any>
    try {
      parsedConfig = JSON.parse(templateForm.rules_config_text)
      if (!parsedConfig || typeof parsedConfig !== 'object' || Array.isArray(parsedConfig)) {
        ElMessage.error('规则配置必须是 JSON 对象')
        return
      }
    } catch (error) {
      console.error('解析规则配置失败:', error)
      ElMessage.error('规则配置需为合法的 JSON 格式')
      return
    }

    const templateName = templateForm.template_name.trim()
    const descriptionValue = templateForm.description.trim()

    saving.value = true
    try {
      if (dialogMode.value === 'create') {
        const payload: CreateRuleTemplateRequest = {
          template_name: templateName,
          rules_config: parsedConfig
        }
        if (descriptionValue) {
          payload.description = descriptionValue
        }
        payload.is_active = templateForm.is_active

        const response = await adminService.createRuleTemplate(payload)
        if (!response.success) {
          throw new Error(response.error?.message || '创建规则模版失败')
        }
        ElMessage.success('规则模版创建成功')
      } else if (editingTemplateId.value) {
        const payload: UpdateRuleTemplateRequest = {
          template_name: templateName,
          is_active: templateForm.is_active,
          rules_config: parsedConfig
        }
        payload.description = descriptionValue

        const response = await adminService.updateRuleTemplate(editingTemplateId.value, payload)
        if (!response.success) {
          throw new Error(response.error?.message || '更新规则模版失败')
        }
        ElMessage.success('规则模版更新成功')
      }

      showTemplateDialog.value = false
      await loadTemplates()
    } catch (error: any) {
      console.error('保存规则模版失败:', error)
      ElMessage.error(error.message || '保存规则模版失败')
    } finally {
      saving.value = false
    }
  })
}

const confirmDelete = async (template: RuleTemplate) => {
  try {
    await ElMessageBox.confirm(
      `确定删除模版"${template.template_name}"吗？删除后无法恢复。`,
      '确认删除',
      {
        confirmButtonText: '确定删除',
        cancelButtonText: '取消',
        type: 'warning'
      }
    )

    const response = await adminService.deleteRuleTemplate(template.id)
    if (!response.success) {
      throw new Error(response.error?.message || '删除规则模版失败')
    }

    ElMessage.success('规则模版删除成功')
    await loadTemplates()
  } catch (error: any) {
    if (error === 'cancel' || error === 'close') {
      return
    }
    console.error('删除规则模版失败:', error)
    ElMessage.error(error.message || '删除规则模版失败')
  }
}

const previewConfig = (template: RuleTemplate) => {
  previewTemplate.value = template
  previewVisible.value = true
}

const handleToggleActive = async (template: RuleTemplate, value: boolean) => {
  if (togglingId.value) {
    return
  }

  const original = template.is_active
  togglingId.value = template.id

  try {
    const response = await adminService.updateRuleTemplate(template.id, {
      is_active: value
    })

    if (!response.success) {
      throw new Error(response.error?.message || '更新规则模版状态失败')
    }

    template.is_active = value
    ElMessage.success(value ? '模版已启用' : '模版已停用')
  } catch (error: any) {
    template.is_active = original
    console.error('更新模版状态失败:', error)
    ElMessage.error(error.message || '更新模版状态失败')
  } finally {
    togglingId.value = null
  }
}

onMounted(() => {
  loadTemplates()
})
</script>

<style scoped>
.admin-rules-page {
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.page-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 16px;
}

.header-left {
  flex: 1;
}

.page-title {
  margin: 0 0 4px 0;
  font-size: 24px;
  font-weight: 600;
  color: #303133;
}

.page-subtitle {
  margin: 0;
  color: #909399;
  font-size: 14px;
}

.header-right {
  display: flex;
  gap: 12px;
  flex-shrink: 0;
}

.filter-bar {
  display: flex;
  align-items: center;
  gap: 16px;
  flex-wrap: wrap;
}

.filter-input {
  flex: 1;
  min-width: 260px;
}

.filter-select {
  width: 160px;
}

.table-card {
  overflow: hidden;
}

.table-card :deep(.el-table__header-wrapper table),
.table-card :deep(.el-table__body-wrapper table) {
  margin: 0 auto;
  display: inline-table;
}

.action-buttons {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-wrap: nowrap;
}

.template-name {
  display: flex;
  align-items: center;
  gap: 8px;
}

.name-text {
  font-weight: 600;
  color: #303133;
}

.json-preview {
  max-height: 480px;
  overflow: auto;
  background: #f5f7fa;
  padding: 16px;
  border-radius: 8px;
  font-family: 'Fira Code', 'Consolas', monospace;
  font-size: 13px;
  line-height: 1.6;
  color: #303133;
  text-align: left;
}

pre {
  margin: 0;
  white-space: pre-wrap;
  word-break: break-word;
}

@media (max-width: 767px) {
  .page-header {
    flex-direction: column;
    align-items: stretch;
  }

  .header-right {
    width: 100%;
    justify-content: flex-end;
  }

  .filter-bar {
    flex-direction: column;
    align-items: stretch;
  }

  .filter-input,
  .filter-select {
    width: 100%;
  }
}
</style>