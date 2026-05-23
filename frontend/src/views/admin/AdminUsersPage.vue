<template>
  <div class="admin-users-page">
    <div class="page-header">
      <div class="header-left">
        <h3 class="page-title">管理员管理</h3>
        <p class="page-subtitle">管理系统管理员账户（仅超级管理员可见）</p>
      </div>
      <div class="header-right">
        <el-button :icon="RefreshRight" @click="fetchAdmins" :loading="loading" circle />
        <el-button type="primary" :icon="Plus" @click="openCreateDialog">
          添加管理员
        </el-button>
      </div>
    </div>

    <el-card>
      <el-alert
        v-if="error"
        type="error"
        :closable="false"
        class="status-alert"
        :title="error"
      />

      <el-table
        v-loading="loading"
        :data="admins"
        border
        empty-text="暂无管理员"
      >
        <el-table-column prop="username" label="用户名" min-width="180" />
        <el-table-column label="账户ID" min-width="260">
          <template #default="{ row }">
            <span class="mono">{{ row.id }}</span>
          </template>
        </el-table-column>
        <el-table-column label="超级管理员" width="140" align="center">
          <template #default="{ row }">
            <el-tag type="success" effect="plain" v-if="row.is_super_admin">
              是
            </el-tag>
            <el-tag type="info" effect="plain" v-else>
              否
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column label="操作" width="220" align="center" fixed="right">
          <template #default="{ row }">
            <el-space wrap>
              <el-button size="small" :icon="Edit" @click="openEditDialog(row)">
                编辑
              </el-button>
              <el-button
                size="small"
                type="danger"
                :icon="Delete"
                @click="confirmDelete(row)"
              >
                删除
              </el-button>
            </el-space>
          </template>
        </el-table-column>
      </el-table>
    </el-card>

    <el-dialog
      v-model="createDialogVisible"
      title="添加管理员"
      width="480px"
      :close-on-click-modal="false"
      :close-on-press-escape="false"
      @closed="resetCreateForm"
    >
      <el-form
        ref="createFormRef"
        :model="createForm"
        :rules="createRules"
        label-width="100px"
      >
        <el-form-item label="用户名" prop="username">
          <el-input v-model="createForm.username" placeholder="请输入用户名" />
        </el-form-item>
        <el-form-item label="登录密码" prop="password">
          <el-input
            v-model="createForm.password"
            type="password"
            show-password
            placeholder="请输入登录密码"
          />
        </el-form-item>
        <el-form-item label="超级管理员">
          <el-switch v-model="createForm.is_super_admin" />
        </el-form-item>
      </el-form>
      <template #footer>
        <span class="dialog-footer">
          <el-button @click="createDialogVisible = false">取消</el-button>
          <el-button type="primary" :loading="submittingCreate" @click="submitCreate">
            确认添加
          </el-button>
        </span>
      </template>
    </el-dialog>

    <el-dialog
      v-model="editDialogVisible"
      title="编辑管理员"
      width="480px"
      :close-on-click-modal="false"
      :close-on-press-escape="false"
      @closed="resetEditForm"
    >
      <el-form
        ref="editFormRef"
        :model="editForm"
        :rules="editRules"
        label-width="100px"
      >
        <el-form-item label="用户名" prop="username">
          <el-input v-model="editForm.username" placeholder="请输入用户名" />
        </el-form-item>
        <el-form-item label="新密码" prop="password">
          <el-input
            v-model="editForm.password"
            type="password"
            show-password
            placeholder="留空则不修改"
          />
        </el-form-item>
        <el-form-item label="超级管理员">
          <el-switch v-model="editForm.is_super_admin" />
        </el-form-item>
      </el-form>
      <template #footer>
        <span class="dialog-footer">
          <el-button @click="editDialogVisible = false">取消</el-button>
          <el-button type="primary" :loading="submittingEdit" @click="submitEdit">
            保存更改
          </el-button>
        </span>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { ElMessage, ElMessageBox } from 'element-plus'
import type { FormInstance, FormRules } from 'element-plus'
import { Plus, RefreshRight, Edit, Delete } from '@element-plus/icons-vue'
import { adminService } from '@/services/adminService'
import type { AdminUser, UpdateAdminRequest } from '@/types/admin'
import { useAdminStore } from '@/stores/admin'

const router = useRouter()
const adminStore = useAdminStore()

const admins = ref<AdminUser[]>([])
const loading = ref(false)
const error = ref<string | null>(null)

const createDialogVisible = ref(false)
const editDialogVisible = ref(false)

const submittingCreate = ref(false)
const submittingEdit = ref(false)

const createFormRef = ref<FormInstance>()
const editFormRef = ref<FormInstance>()

const createForm = reactive({
  username: '',
  password: '',
  is_super_admin: false
})

const editForm = reactive({
  id: '',
  username: '',
  password: '',
  is_super_admin: false
})

const createRules: FormRules = {
  username: [
    { required: true, message: '请输入用户名', trigger: 'blur' },
    { min: 1, max: 40, message: '用户名长度需在 1-40 个字符之间', trigger: 'blur' }
  ],
  password: [
    { required: true, message: '请输入登录密码', trigger: 'blur' },
    { min: 1, max: 40, message: '密码长度需在 1-40 个字符之间', trigger: 'blur' }
  ]
}

const editRules: FormRules = {
  username: [
    { required: true, message: '请输入用户名', trigger: 'blur' },
    { min: 1, max: 40, message: '用户名长度需在 1-40 个字符之间', trigger: 'blur' }
  ],
  password: [
    { min: 1, max: 40, message: '密码长度需在 1-40 个字符之间', trigger: 'blur' }
  ]
}

const extractErrorMessage = (err: any, fallback: string) => {
  return (
    err?.response?.data?.error?.message ||
    err?.response?.data?.message ||
    err?.message ||
    fallback
  )
}

const fetchAdmins = async () => {
  loading.value = true
  error.value = null
  try {
    const data = await adminService.getAdmins()
    admins.value = data
  } catch (err) {
    const message = extractErrorMessage(err, '获取管理员列表失败，请稍后重试')
    error.value = message
    admins.value = []
    ElMessage.error(message)
  } finally {
    loading.value = false
  }
}

const openCreateDialog = () => {
  resetCreateForm()
  createDialogVisible.value = true
}

const resetCreateForm = () => {
  createForm.username = ''
  createForm.password = ''
  createForm.is_super_admin = false
  createFormRef.value?.clearValidate()
}

const submitCreate = () => {
  if (!createFormRef.value) {
    return
  }

  createFormRef.value.validate(async valid => {
    if (!valid) {
      return
    }

    submittingCreate.value = true
    try {
      const response = await adminService.createAdmin({
        username: createForm.username.trim(),
        password: createForm.password,
        is_super_admin: createForm.is_super_admin
      })

      admins.value = [response.user, ...admins.value]
      ElMessage.success(response.message || '管理员创建成功')
      createDialogVisible.value = false
    } catch (err) {
      const message = extractErrorMessage(err, '创建管理员失败，请稍后重试')
      ElMessage.error(message)
    } finally {
      submittingCreate.value = false
    }
  })
}

const openEditDialog = (admin: AdminUser) => {
  editForm.id = admin.id
  editForm.username = admin.username
  editForm.password = ''
  editForm.is_super_admin = admin.is_super_admin
  editFormRef.value?.clearValidate()
  editDialogVisible.value = true
}

const resetEditForm = () => {
  editForm.id = ''
  editForm.username = ''
  editForm.password = ''
  editForm.is_super_admin = false
  editFormRef.value?.clearValidate()
}

const submitEdit = () => {
  if (!editFormRef.value) {
    return
  }

  editFormRef.value.validate(async valid => {
    if (!valid) {
      return
    }

    submittingEdit.value = true
    try {
      const payload: UpdateAdminRequest = {
        username: editForm.username.trim(),
        is_super_admin: editForm.is_super_admin
      }

      if (editForm.password.trim().length > 0) {
        payload.password = editForm.password
      }

      const response = await adminService.updateAdmin(editForm.id, payload)

      admins.value = admins.value.map(item =>
        item.id === response.user.id ? { ...item, ...response.user } : item
      )

      if (adminStore.userInfo?.id === response.user.id) {
        adminStore.userInfo = response.user
        localStorage.setItem('admin_user', JSON.stringify(response.user))

        if (!response.user.is_super_admin) {
          ElMessage.warning('当前账号已不再是超级管理员，即将返回游戏管理页')
          router.push('/admin/games')
        }
      }

      ElMessage.success(response.message || '管理员信息已更新')
      editDialogVisible.value = false
    } catch (err) {
      const message = extractErrorMessage(err, '更新管理员失败，请稍后重试')
      ElMessage.error(message)
    } finally {
      submittingEdit.value = false
    }
  })
}

const confirmDelete = async (admin: AdminUser) => {
  try {
    await ElMessageBox.confirm(
      `确定删除管理员“${admin.username}”吗？该操作不可撤销。`,
      '删除确认',
      {
        type: 'warning',
        confirmButtonText: '确认删除',
        cancelButtonText: '取消'
      }
    )

    const response = await adminService.deleteAdmin(admin.id)
    admins.value = admins.value.filter(item => item.id !== admin.id)
    ElMessage.success(response.message || '管理员已删除')

    if (adminStore.userInfo?.id === admin.id) {
      adminStore.logout()
      ElMessage.warning('当前登录账号已被删除，已退出登录')
      router.push('/admin/login')
    }
  } catch (err) {
    if (err === 'cancel' || err === 'close') {
      return
    }

    const message = extractErrorMessage(err, '删除管理员失败，请稍后重试')
    ElMessage.error(message)
  }
}

onMounted(() => {
  fetchAdmins()
})
</script>

<style scoped>
.admin-users-page {
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
  align-items: center;
  gap: 12px;
}

.status-alert {
  margin-bottom: 16px;
}

.mono {
  font-family: 'Fira Code', 'Consolas', 'Courier New', monospace;
  font-size: 13px;
  color: #606266;
}

.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}

.admin-users-page :deep(.el-table__header-wrapper table),
.admin-users-page :deep(.el-table__body-wrapper table) {
  margin: 0 auto;
  display: inline-table;
}

@media (max-width: 767px) {
  .page-header {
    flex-direction: column;
    align-items: stretch;
  }

  .header-right {
    justify-content: flex-end;
  }
}
</style>