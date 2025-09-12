<template>
  <div class="director-console">
    <!-- 页面头部 -->
    <div class="console-header">
      <div class="header-content">
        <h2>导演控制台</h2>
        <p class="game-info">游戏ID: {{ $route.params.id }}</p>
        <el-button 
          @click="goBack" 
          :icon="ArrowLeft"
          class="back-button"
        >
          返回游戏详情
        </el-button>
      </div>
    </div>

    <!-- 导演控制台主界面 -->
    <div v-if="!state.passwordFromURI || state.isAuthenticated" class="console-main">
      <!-- 认证区域 -->
      <el-card v-if="!state.passwordFromURI && !state.isAuthenticated" class="auth-card">
        <template #header>
          <div class="card-header">
            <h3>导演身份验证</h3>
          </div>
        </template>
        
        <div class="auth-form">
          <el-alert 
            v-if="state.authError" 
            :title="state.authError" 
            type="error" 
            show-icon 
            :closable="false"
            class="auth-error"
          />
          
          <el-form @submit.prevent="handleAuthenticate">
            <el-form-item label="导演密码">
              <el-input
                v-model="state.directorPassword"
                type="password"
                placeholder="请输入导演密码"
                show-password
                :disabled="state.authLoading"
                @keyup.enter="handleAuthenticate"
              />
            </el-form-item>
            
            <el-form-item>
              <el-button 
                type="primary" 
                @click="handleAuthenticate"
                :loading="state.authLoading"
                style="width: 100%"
              >
                验证身份
              </el-button>
            </el-form-item>
          </el-form>
        </div>
      </el-card>
      <!-- 演员管理区域 -->
      <el-card class="players-card">
        <template #header>
          <div class="card-header">
            <div class="header-left">
              <el-button
                :icon="state.playersTableCollapsed ? ArrowDown : ArrowUp"
                @click="state.playersTableCollapsed = !state.playersTableCollapsed"
                text
                class="collapse-btn"
              >
                演员管理列表 ({{ state.players.length }}人)
              </el-button>
            </div>
            <div class="header-right">
              <el-button
                type="primary"
                :icon="Plus"
                @click="openBatchAddDialog"
              >
                批量添加演员
              </el-button>
              <el-button
                v-if="state.selectedPlayers.length > 0"
                type="danger"
                :icon="Delete"
                @click="openBatchDeleteDialog"
                :loading="state.batchDeleteLoading"
              >
                批量删除 ({{ state.selectedPlayers.length }})
              </el-button>
              <el-button
                :icon="Refresh"
                @click="refreshPlayers"
                :loading="state.playersLoading"
              >
                刷新
              </el-button>
            </div>
          </div>
        </template>

        <!-- 演员表格 -->
        <el-collapse-transition>
          <div v-show="!state.playersTableCollapsed">
            <el-table
              :data="state.players"
              v-loading="state.playersLoading"
              @selection-change="handleSelectionChange"
              stripe
              class="players-table"
            >
              <el-table-column type="selection" width="55" />
              
              <el-table-column prop="id" label="演员ID" width="280">
                <template #default="{ row }">
                  <el-text class="player-id">{{ row.id }}</el-text>
                </template>
              </el-table-column>
              
              <el-table-column prop="name" label="演员名称" min-width="120">
                <template #default="{ row }">
                  <el-text class="player-name">{{ row.name }}</el-text>
                </template>
              </el-table-column>
              
              <el-table-column prop="password" label="登录密码" width="120">
                <template #default="{ row }">
                  <el-text class="player-password">{{ row.password }}</el-text>
                </template>
              </el-table-column>
              
              <el-table-column prop="team_id" label="组队编号" width="100">
                <template #default="{ row }">
                  <el-tag v-if="row.team_id > 0" size="small">{{ row.team_id }}</el-tag>
                  <el-text v-else class="no-team">无</el-text>
                </template>
              </el-table-column>
              
              <el-table-column label="操作" width="120" fixed="right">
                <template #default="{ row }">
                  <el-button
                    size="small"
                    type="primary"
                    @click="switchToPlayerView(row.id)"
                    :icon="View"
                  >
                    进入视角
                  </el-button>
                </template>
              </el-table-column>
            </el-table>
            
            <!-- 空状态 -->
            <el-empty 
              v-if="state.players.length === 0 && !state.playersLoading"
              description="暂无演员数据"
              :image-size="80"
            />
          </div>
        </el-collapse-transition>
      </el-card>
    </div>

    <!-- 批量添加演员对话框 -->
    <el-dialog
      v-model="state.batchAddDialogVisible"
      title="批量添加演员账户"
      width="800px"
      :close-on-click-modal="false"
      destroy-on-close
    >
      <div class="batch-add-content">
        <el-alert
          title="批量粘贴说明"
          type="info"
          show-icon
          :closable="false"
          class="batch-info"
        >
          <template #default>
            <p>请在下方三个输入框中分别粘贴用户名、密码和组队编号（每行一个）：</p>
            <ul>
              <li>用户名和密码必须一一对应，数量必须相等</li>
              <li>密码要求：6-8位字母数字组合</li>
              <li>组队编号可选，可以少于用户名数量</li>
            </ul>
          </template>
        </el-alert>
        
        <div class="paste-form">
          <div class="paste-column">
            <label class="paste-label">用户名列表</label>
            <el-input
              v-model="state.pasteUsernames"
              type="textarea"
              :rows="8"
              placeholder="每行一个用户名&#10;示例：&#10;张三&#10;李四&#10;王五"
              @input="validatePasteData"
            />
          </div>
          
          <div class="paste-column">
            <label class="paste-label">密码列表</label>
            <el-input
              v-model="state.pastePasswords"
              type="textarea"
              :rows="8"
              placeholder="每行一个密码&#10;示例：&#10;abc123&#10;def456&#10;ghi789"
              @input="validatePasteData"
            />
          </div>
          
          <div class="paste-column">
            <label class="paste-label">组队编号（可选）</label>
            <el-input
              v-model="state.pasteTeamIds"
              type="textarea"
              :rows="8"
              placeholder="每行一个组队编号&#10;示例：&#10;1&#10;2&#10;3&#10;（可留空或少于用户名数量）"
              @input="validatePasteData"
            />
          </div>
        </div>
        
        <!-- 预览数据 -->
        <div v-if="pastePreviewData.length > 0" class="paste-preview">
          <h4>预览数据 ({{ pastePreviewData.length }}条)</h4>
          <el-table :data="pastePreviewData" size="small" max-height="200">
            <el-table-column prop="player_name" label="用户名" width="150" align="center" header-align="center" />
            <el-table-column prop="password" label="密码" width="120" align="center" header-align="center" />
            <el-table-column prop="team_id" label="组队编号" width="100" align="center" header-align="center">
              <template #default="{ row }">
                <el-tag v-if="row.team_id && row.team_id > 0" size="small">{{ row.team_id }}</el-tag>
                <el-text v-else class="no-team">无</el-text>
              </template>
            </el-table-column>
          </el-table>
        </div>
        
        <!-- 错误提示 -->
        <el-alert
          v-if="pasteErrorMessage"
          :title="pasteErrorMessage"
          type="error"
          show-icon
          :closable="false"
          class="paste-error"
        />
      </div>
      
      <template #footer>
        <div class="dialog-footer">
          <el-button @click="closeBatchAddDialog">取消</el-button>
          <el-button
            type="primary"
            @click="handleBatchAdd"
            :disabled="pastePreviewData.length === 0 || !!pasteErrorMessage"
            :loading="state.batchAddLoading"
          >
            批量添加 ({{ pastePreviewData.length }}条)
          </el-button>
        </div>
      </template>
    </el-dialog>

    <!-- 批量删除确认对话框 -->
    <el-dialog
      v-model="batchDeleteDialogVisible"
      title="确认删除演员"
      width="500px"
      :close-on-click-modal="false"
    >
      <div class="delete-confirm-content">
        <el-alert
          title="警告"
          type="warning"
          show-icon
          :closable="false"
          class="delete-warning"
        >
          您即将删除以下演员账户，此操作不可恢复！
        </el-alert>
        
        <div class="selected-players">
          <h4>将要删除的演员：</h4>
          <ul>
            <li v-for="player in selectedPlayersData" :key="player.id">
              {{ player.name }} (ID: {{ player.id }})
            </li>
          </ul>
        </div>
        
        <el-alert
          title="注意"
          type="info"
          show-icon
          :closable="false"
        >
          游戏开始后不允许删除演员
        </el-alert>
      </div>
      
      <template #footer>
        <div class="dialog-footer">
          <el-button @click="batchDeleteDialogVisible = false">取消</el-button>
          <el-button
            type="danger"
            @click="handleBatchDelete"
            :loading="state.batchDeleteLoading"
          >
            确认删除
          </el-button>
        </div>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted, watch } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { ElMessage, ElMessageBox } from 'element-plus'
import {
  ArrowLeft,
  ArrowDown,
  ArrowUp,
  Plus,
  Delete,
  Refresh,
  View
} from '@element-plus/icons-vue'

import { directorService } from '@/services/directorService'
import type {
  DirectorConsoleState,
  PlayerInfo,
  BatchPasteData
} from '@/types/director'

const router = useRouter()
const route = useRoute()

// 响应式状态
const state = reactive<DirectorConsoleState>({
  // URI参数相关
  passwordFromURI: null,
  autoAuthenticated: false,
  
  // 认证相关
  isAuthenticated: false,
  directorPassword: '',
  authLoading: false,
  authError: null,
  
  // 演员数据
  players: [],
  playersLoading: false,
  
  // UI状态
  playersTableCollapsed: false,
  
  // 批量操作状态
  selectedPlayers: [],
  batchAddDialogVisible: false,
  batchDeleteLoading: false,
  batchAddLoading: false,
  
  // 批量添加表单
  addPlayersForm: [],
  
  // 批量粘贴功能
  batchPasteDialogVisible: false,
  pasteUsernames: '',
  pastePasswords: '',
  pasteTeamIds: ''
})

// 批量删除对话框状态
const batchDeleteDialogVisible = ref(false)

// 粘贴数据验证相关
const pasteValidationResult = ref<BatchPasteData>({
  usernames: [],
  passwords: [],
  teamIds: [],
  isValid: false
})

// 计算属性
const gameId = computed(() => route.params.id as string)

const selectedPlayersData = computed(() => {
  return state.players.filter(player => state.selectedPlayers.includes(player.id))
})

const pastePreviewData = computed(() => {
  if (!pasteValidationResult.value.isValid) return []
  
  return pasteValidationResult.value.usernames.map((username, index) => ({
    player_name: username,
    password: pasteValidationResult.value.passwords[index],
    team_id: pasteValidationResult.value.teamIds[index] ? parseInt(pasteValidationResult.value.teamIds[index]) || 0 : 0,
    tempId: directorService.generateTempId()
  }))
})

const pasteErrorMessage = computed(() => {
  return pasteValidationResult.value.errorMessage || null
})

// 生命周期和监听器
onMounted(async () => {
  // 检查是否从URI中获取密码
  checkURIPassword()
  
  // 如果有URI密码，直接进行认证，跳过输入界面
  if (state.passwordFromURI) {
    state.directorPassword = state.passwordFromURI
    state.isAuthenticated = true // 直接设置为已认证
    await attemptAuthentication()
  }
})

// 监听路由变化
watch(() => route.fullPath, () => {
  checkURIPassword()
})

// 方法实现
const checkURIPassword = () => {
  const password = directorService.parsePasswordFromURI(route.fullPath)
  if (password) {
    state.passwordFromURI = password
    state.autoAuthenticated = false
  }
}

const attemptAuthentication = async () => {
  if (!state.directorPassword.trim()) {
    state.authError = '请输入导演密码'
    return
  }
  
  state.authLoading = true
  state.authError = null
  
  try {
    const response = await directorService.authenticateAndGetPlayers(
      gameId.value,
      state.directorPassword
    )
    
    if (response.success && response.data) {
      state.isAuthenticated = true
      state.players = response.data.players
      state.autoAuthenticated = !!state.passwordFromURI
      
      ElMessage.success('导演身份验证成功')
    } else {
      throw new Error('认证失败')
    }
  } catch (error: any) {
    console.error('导演认证失败:', error)
    
    if (error.response?.status === 401) {
      state.authError = '导演密码错误'
    } else if (error.response?.status === 404) {
      state.authError = '游戏不存在'
    } else {
      state.authError = '认证失败，请稍后重试'
    }
    
    state.isAuthenticated = false
  } finally {
    state.authLoading = false
  }
}

const handleAuthenticate = async () => {
  await attemptAuthentication()
}

const refreshPlayers = async () => {
  if (!state.isAuthenticated) return
  
  state.playersLoading = true
  
  try {
    const response = await directorService.authenticateAndGetPlayers(
      gameId.value,
      state.directorPassword
    )
    
    if (response.success && response.data) {
      state.players = response.data.players
      ElMessage.success('演员列表刷新成功')
    }
  } catch (error) {
    console.error('刷新演员列表失败:', error)
    ElMessage.error('刷新失败，请稍后重试')
  } finally {
    state.playersLoading = false
  }
}

const handleSelectionChange = (selection: PlayerInfo[]) => {
  state.selectedPlayers = selection.map(player => player.id)
}

const openBatchAddDialog = () => {
  state.batchAddDialogVisible = true
  state.pasteUsernames = ''
  state.pastePasswords = ''
  state.pasteTeamIds = ''
  pasteValidationResult.value = {
    usernames: [],
    passwords: [],
    teamIds: [],
    isValid: false
  }
}

const closeBatchAddDialog = () => {
  state.batchAddDialogVisible = false
}

const validatePasteData = () => {
  // 首先进行基本数据验证
  const basicValidation = directorService.parseBatchPasteData(
    state.pasteUsernames,
    state.pastePasswords,
    state.pasteTeamIds
  )
  
  if (!basicValidation.isValid) {
    pasteValidationResult.value = basicValidation
    return
  }
  
  // 检查与现有用户的重复
  const duplicateCheck = directorService.checkDuplicatesWithExistingUsers(
    basicValidation.usernames,
    basicValidation.passwords,
    state.players
  )
  
  if (!duplicateCheck.isValid) {
    pasteValidationResult.value = {
      ...basicValidation,
      isValid: false,
      errorMessage: duplicateCheck.errorMessage
    }
    return
  }
  
  // 所有验证通过
  pasteValidationResult.value = basicValidation
}

const handleBatchAdd = async () => {
  if (!pasteValidationResult.value.isValid || pastePreviewData.value.length === 0) {
    ElMessage.error('数据验证失败，请检查输入')
    return
  }
  
  state.batchAddLoading = true
  
  try {
    const response = await directorService.batchAddPlayers(
      gameId.value,
      state.directorPassword,
      pastePreviewData.value
    )
    
    if (response.success && response.data) {
      const { success, failed } = response.data
      
      // 显示结果
      if (success.length > 0) {
        ElMessage.success(`成功添加 ${success.length} 个演员`)
      }
      
      if (failed.length > 0) {
        const failedMessages = failed.map(f => `${f.player_name || '未知'}: ${f.reason}`).join('\n')
        ElMessageBox.alert(failedMessages, '部分演员添加失败', {
          type: 'warning'
        })
      }
      
      // 刷新列表
      await refreshPlayers()
      
      // 关闭对话框
      closeBatchAddDialog()
    }
  } catch (error: any) {
    console.error('批量添加演员失败:', error)
    
    if (error.response?.status === 401) {
      ElMessage.error('导演密码错误')
    } else {
      ElMessage.error('添加失败，请稍后重试')
    }
  } finally {
    state.batchAddLoading = false
  }
}

const openBatchDeleteDialog = () => {
  if (state.selectedPlayers.length === 0) {
    ElMessage.warning('请先选择要删除的演员')
    return
  }
  
  batchDeleteDialogVisible.value = true
}

const handleBatchDelete = async () => {
  if (state.selectedPlayers.length === 0) {
    ElMessage.warning('请先选择要删除的演员')
    return
  }
  
  state.batchDeleteLoading = true
  
  try {
    const response = await directorService.batchDeletePlayers(
      gameId.value,
      state.directorPassword,
      state.selectedPlayers
    )
    
    if (response.success && response.data) {
      const { success, failed } = response.data
      
      // 显示结果
      if (success.length > 0) {
        ElMessage.success(`成功删除 ${success.length} 个演员`)
      }
      
      if (failed.length > 0) {
        const failedMessages = failed.map(f => `${f.id || '未知'}: ${f.reason}`).join('\n')
        ElMessageBox.alert(failedMessages, '部分演员删除失败', {
          type: 'warning'
        })
      }
      
      // 清空选择并刷新列表
      state.selectedPlayers = []
      await refreshPlayers()
      
      // 关闭对话框
      batchDeleteDialogVisible.value = false
    }
  } catch (error: any) {
    console.error('批量删除演员失败:', error)
    
    if (error.response?.status === 401) {
      ElMessage.error('导演密码错误')
    } else if (error.response?.status === 400) {
      ElMessage.error('游戏已开始，无法删除演员')
    } else {
      ElMessage.error('删除失败，请稍后重试')
    }
  } finally {
    state.batchDeleteLoading = false
  }
}

const switchToPlayerView = (_playerId: string) => {
  // 跳转到指定演员视角 - 可以根据需要实现
  router.push(`/game/${gameId.value}/player`)
}

const goBack = () => {
  router.push(`/game/${gameId.value}`)
}
</script>

<style scoped>
.director-console {
  min-height: 100vh;
  padding: 24px;
  background-color: #f5f7fa;
}

/* 页面头部样式 */
.console-header {
  margin-bottom: 24px;
  background: white;
  border-radius: 8px;
  padding: 20px;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
}

.header-content {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.header-content h2 {
  margin: 0;
  color: #303133;
  font-size: 24px;
}

.game-info {
  margin: 8px 0 0 0;
  color: #909399;
  font-size: 14px;
}

.back-button {
  margin-left: auto;
}

/* 认证卡片样式 */
.auth-card {
  max-width: 400px;
  margin: 0 auto;
}

.auth-form {
  padding: 20px 0;
}

.auth-error {
  margin-bottom: 20px;
}

/* 控制台主界面 */
.console-main {
  max-width: 1200px;
  margin: 0 auto;
}

/* 卡片头部 */
.card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.header-left {
  display: flex;
  align-items: center;
}

.header-right {
  display: flex;
  gap: 12px;
}

.collapse-btn {
  font-size: 16px;
  font-weight: 600;
  color: #303133;
}

/* 演员表格样式 */
.players-card {
  margin-bottom: 24px;
  min-width: 900px; /* 设置最小宽度以保持布局稳定 */
}

.players-table {
  margin-top: 16px;
}

.player-id {
  font-family: 'Monaco', 'Consolas', monospace;
  font-size: 12px;
  color: #606266;
}

.player-name {
  font-weight: 500;
  color: #303133;
}

.player-password {
  font-family: 'Monaco', 'Consolas', monospace;
  font-weight: 600;
  color: #E6A23C;
  background: #FDF6EC;
  padding: 4px 8px;
  border-radius: 4px;
  border: 1px solid #F5DAB1;
}

.no-team {
  color: #C0C4CC;
  font-style: italic;
}

/* 批量添加对话框 */
.batch-add-content {
  max-height: 70vh;
  overflow-y: auto;
}

.batch-info {
  margin-bottom: 20px;
}

.batch-info ul {
  margin: 8px 0 0 20px;
  padding: 0;
}

.batch-info li {
  margin-bottom: 4px;
  color: #606266;
}

.paste-form {
  display: grid;
  grid-template-columns: 1fr 1fr 1fr;
  gap: 16px;
  margin-bottom: 20px;
}

.paste-column {
  display: flex;
  flex-direction: column;
}

.paste-column label {
  font-weight: 600;
  color: #303133;
  margin-bottom: 8px;
  font-size: 14px;
  text-align: center;
  display: block;
}

.paste-label {
  font-weight: 600;
  color: #303133;
  margin-bottom: 8px;
  font-size: 14px;
  text-align: center;
  display: block;
}

.paste-preview {
  margin-bottom: 20px;
  padding: 16px;
  background: #F5F7FA;
  border-radius: 6px;
  border: 1px solid #DCDFE6;
}

.paste-preview h4 {
  margin: 0 0 12px 0;
  color: #303133;
  font-size: 14px;
}

/* 预览表格居中对齐 */
.paste-preview :deep(.el-table__header-wrapper table) {
  margin: 0 auto !important;
  display: inline-table !important;
}

.paste-preview :deep(.el-table__body-wrapper table) {
  margin: 0 auto !important;
  display: inline-table !important;
}

.paste-error {
  margin-bottom: 20px;
}

/* 批量删除对话框 */
.delete-confirm-content {
  padding: 0 4px;
}

.delete-warning {
  margin-bottom: 20px;
}

.selected-players {
  margin: 20px 0;
  padding: 16px;
  background: #FDF6EC;
  border-radius: 6px;
  border: 1px solid #F5DAB1;
}

.selected-players h4 {
  margin: 0 0 12px 0;
  color: #E6A23C;
  font-size: 14px;
}

.selected-players ul {
  margin: 0;
  padding: 0;
  list-style: none;
}

.selected-players li {
  padding: 4px 0;
  color: #606266;
  border-bottom: 1px solid #F5DAB1;
}

.selected-players li:last-child {
  border-bottom: none;
}

/* 对话框底部 */
.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
}

/* 响应式设计 */
@media (max-width: 768px) {
  .director-console {
    padding: 16px;
  }
  
  .header-content {
    flex-direction: column;
    align-items: flex-start;
    gap: 12px;
  }
  
  .back-button {
    margin-left: 0;
    align-self: flex-end;
  }
  
  .card-header {
    flex-direction: column;
    align-items: flex-start;
    gap: 12px;
  }
  
  .header-right {
    width: 100%;
    justify-content: flex-start;
    flex-wrap: wrap;
  }
  
  .paste-form {
    grid-template-columns: 1fr;
    gap: 16px;
  }
  
  .console-main {
    margin: 0;
  }
}

@media (max-width: 480px) {
  .director-console {
    padding: 12px;
  }
  
  .console-header {
    padding: 16px;
  }
  
  .header-content h2 {
    font-size: 20px;
  }
}

/* 动画效果 */
.el-collapse-transition {
  transition: height 0.3s ease;
}

/* 自定义滚动条 */
.batch-add-content::-webkit-scrollbar {
  width: 6px;
}

.batch-add-content::-webkit-scrollbar-track {
  background: #f1f1f1;
  border-radius: 3px;
}

.batch-add-content::-webkit-scrollbar-thumb {
  background: #c1c1c1;
  border-radius: 3px;
}

.batch-add-content::-webkit-scrollbar-thumb:hover {
  background: #a8a8a8;
}
</style>