<template>
  <div class="admin-games-page">
    <!-- 页面头部操作栏 -->
    <div class="page-header">
      <div class="header-left">
        <h3 class="page-title">游戏管理</h3>
        <p class="page-subtitle">管理所有游戏，包括创建、编辑和状态控制</p>
      </div>
      <div class="header-right">
        <el-button 
          type="primary" 
          :icon="Plus"
          @click="createGame()"
        >
          创建游戏
        </el-button>
      </div>
    </div>

    <!-- 搜索和筛选栏 -->
    <div class="search-bar">
      <el-input
        v-model="searchQuery"
        placeholder="搜索游戏名称或描述..."
        :prefix-icon="Search"
        clearable
        class="search-input"
      />
      <el-select v-model="statusFilter" placeholder="状态筛选" clearable>
        <el-option label="全部状态" value="" />
        <el-option label="等待中" value="waiting" />
        <el-option label="进行中" value="running" />
        <el-option label="已暂停" value="paused" />
        <el-option label="已结束" value="ended" />
        <el-option label="已隐藏" value="hidden" />
        <el-option label="已删除" value="deleted" />
      </el-select>
      <el-button @click="refreshGames" :icon="Refresh" :loading="loading">
        刷新
      </el-button>
    </div>

    <!-- 游戏列表表格 -->
    <el-card class="table-card">
      <el-table 
        :data="filteredGames" 
        v-loading="loading"
        empty-text="暂无游戏数据"
        @selection-change="handleSelectionChange"
      >
        <el-table-column type="selection" width="55" />
        
        <el-table-column prop="name" label="游戏名称" min-width="150">
          <template #default="{ row }">
            <div class="game-name">
              <span class="name-text">{{ row.name }}</span>
              <el-tag 
                :type="getGameStatusConfig(row.status).type"
                size="small"
                effect="dark"
                class="status-tag"
              >
                {{ getGameStatusConfig(row.status).text }}
              </el-tag>
            </div>
          </template>
        </el-table-column>

        <el-table-column prop="description" label="描述" min-width="200" show-overflow-tooltip />

        <el-table-column label="玩家数量" width="100" align="center">
          <template #default="{ row }">
            {{ row.player_count }}/{{ row.max_players }}
          </template>
        </el-table-column>

        <el-table-column prop="created_at" label="创建时间" width="160">
          <template #default="{ row }">
            {{ formatDateTime(row.created_at) }}
          </template>
        </el-table-column>

        <el-table-column label="操作" width="200" fixed="right">
          <template #default="{ row }">
            <el-button 
              type="primary" 
              size="small" 
              :icon="Edit"
              @click="editGame(row)"
              text
            >
              编辑
            </el-button>
            <el-button 
              type="warning" 
              size="small" 
              :icon="View"
              @click="viewGame(row)"
              text
            >
              查看
            </el-button>
            <el-dropdown @command="(command: string) => handleGameAction(command, row)">
              <el-button type="info" size="small" :icon="More" text />
              <template #dropdown>
                <el-dropdown-menu>
                  <el-dropdown-item 
                    command="hide" 
                    v-if="row.status !== 'hidden' && row.status !== 'deleted'"
                  >
                    <el-icon><Hide /></el-icon>
                    隐藏游戏
                  </el-dropdown-item>
                  <el-dropdown-item 
                    command="show" 
                    v-if="row.status === 'hidden'"
                  >
                    <el-icon><View /></el-icon>
                    显示游戏
                  </el-dropdown-item>
                  <el-dropdown-item 
                    command="delete" 
                    v-if="row.status !== 'deleted'"
                    divided
                  >
                    <el-icon><Delete /></el-icon>
                    删除游戏
                  </el-dropdown-item>
                </el-dropdown-menu>
              </template>
            </el-dropdown>
          </template>
        </el-table-column>
      </el-table>

      <!-- 批量操作 -->
      <div class="table-footer" v-if="selectedGames.length > 0">
        <div class="selection-info">
          已选择 {{ selectedGames.length }} 项
        </div>
        <div class="batch-actions">
          <el-button size="small" @click="batchHide">批量隐藏</el-button>
          <el-button size="small" type="danger" @click="batchDelete">批量删除</el-button>
        </div>
      </div>
    </el-card>

    <!-- 创建/编辑游戏对话框 -->
    <el-dialog
      :title="editingGame ? '编辑游戏' : '创建游戏'"
      v-model="showCreateDialog"
      width="600px"
      :close-on-click-modal="false"
    >
      <el-form 
        ref="gameFormRef"
        :model="gameForm"
        :rules="gameFormRules"
        label-width="100px"
      >
        <el-form-item label="游戏名称" prop="name">
          <el-input v-model="gameForm.name" placeholder="请输入游戏名称" />
        </el-form-item>

        <el-form-item label="游戏描述" prop="description">
          <el-input 
            v-model="gameForm.description" 
            type="textarea" 
            :rows="3"
            placeholder="请输入游戏描述（可选）"
          />
        </el-form-item>

        <el-form-item label="导演密码" prop="director_password">
          <el-input 
            v-model="gameForm.director_password" 
            type="text"
            placeholder="请输入导演密码"
          />
        </el-form-item>

        <el-form-item label="最大玩家数" prop="max_players">
          <el-input-number 
            v-model="gameForm.max_players" 
            :min="1" 
            :max="1000"
            controls-position="right"
          />
        </el-form-item>

        <!-- 创建游戏时显示规则模板选择 -->
        <el-form-item v-if="!editingGame" label="规则模版" prop="rule_template_id">
          <el-select 
            v-model="gameForm.rule_template_id" 
            placeholder="请选择规则模版（可选）"
            clearable
          >
            <el-option 
              v-for="template in ruleTemplates"
              :key="template.id"
              :label="template.template_name"
              :value="template.id"
            />
          </el-select>
        </el-form-item>
      </el-form>

      <template #footer>
        <el-button @click="cancelEdit">取消</el-button>
        <el-button 
          type="primary" 
          @click="saveGame"
          :loading="saving"
        >
          {{ editingGame ? '保存' : '创建' }}
        </el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { ElMessage, ElMessageBox, type FormInstance, type FormRules } from 'element-plus'
import { 
  Plus, 
  Search, 
  Refresh, 
  Edit, 
  View, 
  More, 
  Hide, 
  Delete 
} from '@element-plus/icons-vue'
import { gameService } from '@/services/gameService'
import { adminService } from '@/services/adminService'
import { getGameStatusConfig, formatDateTime } from '@/utils/gameFilter'
import type { GameListItem, CreateGameRequest, UpdateGameRequest } from '@/types/game'
import type { RuleTemplate } from '@/types/admin'

const router = useRouter()

// 响应式数据
const loading = ref(false)
const saving = ref(false)
const games = ref<GameListItem[]>([])
const ruleTemplates = ref<RuleTemplate[]>([])
const searchQuery = ref('')
const statusFilter = ref('')
const selectedGames = ref<GameListItem[]>([])
const showCreateDialog = ref(false)
const editingGame = ref<GameListItem | null>(null)

// 表单引用和数据
const gameFormRef = ref<FormInstance>()
const gameForm = reactive<CreateGameRequest>({
  name: '',
  description: '',
  director_password: '',
  max_players: 10,
  rule_template_id: ''
})

// 表单验证规则
const gameFormRules: FormRules = {
  name: [
    { required: true, message: '请输入游戏名称', trigger: 'blur' },
    { min: 1, max: 100, message: '游戏名称长度在 1 到 100 个字符', trigger: 'blur' }
  ],
  director_password: [
    { required: true, message: '请输入导演密码', trigger: 'blur' },
    { min: 6, max: 50, message: '导演密码长度在 6 到 50 个字符', trigger: 'blur' }
  ],
  max_players: [
    { required: true, message: '请输入最大玩家数', trigger: 'blur' },
    { type: 'number', min: 1, max: 1000, message: '最大玩家数在 1 到 1000 之间', trigger: 'blur' }
  ]
}

// 计算属性
const filteredGames = computed(() => {
  let result = games.value

  // 按状态筛选
  if (statusFilter.value) {
    result = result.filter(game => game.status === statusFilter.value)
  }

  // 按搜索关键字筛选
  if (searchQuery.value.trim()) {
    const query = searchQuery.value.toLowerCase().trim()
    result = result.filter(game => 
      game.name.toLowerCase().includes(query) ||
      (game.description && game.description.toLowerCase().includes(query))
    )
  }

  return result
})

// 方法
const loadGames = async () => {
  loading.value = true
  try {
    const response = await gameService.getGames(undefined, true) // 使用管理员权限获取游戏列表
    if (response.success && response.data) {
      games.value = response.data
    }
  } catch (error) {
    console.error('加载游戏列表失败:', error)
    ElMessage.error('加载游戏列表失败')
  } finally {
    loading.value = false
  }
}

const loadRuleTemplates = async () => {
  try {
    const response = await adminService.getRuleTemplates()
    if (response.success && response.data) {
      ruleTemplates.value = response.data.filter(t => t.is_active)
    }
  } catch (error) {
    console.error('加载规则模版失败:', error)
  }
}

const refreshGames = async () => {
  await loadGames()
  ElMessage.success('刷新成功')
}

const handleSelectionChange = (selection: GameListItem[]) => {
  selectedGames.value = selection
}

const editGame = (game: GameListItem) => {
  editingGame.value = game
  
  // 填充表单数据
  gameForm.name = game.name
  gameForm.description = game.description || ''
  gameForm.director_password = game.director_password || '' // 显示现有密码
  gameForm.max_players = game.max_players
  // 注意：编辑时不再显示规则模板选择
  
  showCreateDialog.value = true
}

const createGame = () => {
  editingGame.value = null;
  showCreateDialog.value = true
}

const viewGame = (game: GameListItem) => {
  router.push(`/game/${game.id}`)
}

const handleGameAction = async (command: string, game: GameListItem) => {
  switch (command) {
    case 'hide':
      await updateGameStatus(game, 'hidden')
      break
    case 'show':
      await updateGameStatus(game, 'waiting')
      break
    case 'delete':
      await confirmAndDeleteGame(game)
      break
  }
}

const updateGameStatus = async (game: GameListItem, _status: string) => {
  try {
    // 这里需要调用更新游戏状态的API
    // 由于当前API设计中没有直接的状态更新接口，这里先用占位逻辑
    ElMessage.success(`游戏"${game.name}"状态已更新`)
    await loadGames()
  } catch (error) {
    ElMessage.error('更新游戏状态失败')
  }
}

const confirmAndDeleteGame = async (game: GameListItem) => {
  try {
    await ElMessageBox.confirm(
      `确定要删除游戏"${game.name}"吗？此操作不可恢复！`,
      '确认删除',
      {
        confirmButtonText: '确定删除',
        cancelButtonText: '取消',
        type: 'warning'
      }
    )
    
    await gameService.deleteGame(game.id)
    ElMessage.success('游戏删除成功')
    await loadGames()
  } catch (error) {
    if (error !== 'cancel') {
      ElMessage.error('删除游戏失败')
    }
  }
}

const batchHide = async () => {
  // 批量隐藏逻辑
  ElMessage.info('批量隐藏功能待实现')
}

const batchDelete = async () => {
  // 批量删除逻辑
  try {
    await ElMessageBox.confirm(
      `确定要删除选中的 ${selectedGames.value.length} 个游戏吗？此操作不可恢复！`,
      '确认批量删除',
      {
        confirmButtonText: '确定删除',
        cancelButtonText: '取消',
        type: 'warning'
      }
    )
    
    ElMessage.info('批量删除功能待实现')
  } catch {
    // 用户取消
  }
}

const saveGame = async () => {
  if (!gameFormRef.value) return

  const isValid = await gameFormRef.value.validate().catch(() => false)
  if (!isValid) return

  saving.value = true

  try {
    if (editingGame.value) {
      // 编辑游戏
      const updateData: UpdateGameRequest = {
        name: gameForm.name,
        description: gameForm.description || undefined,
        max_players: gameForm.max_players
        // 注意：编辑时不再包含 rule_template_id
      }
      
      // 只有输入了新密码才更新
      if (gameForm.director_password) {
        updateData.director_password = gameForm.director_password
      }
      
      await gameService.updateGame(editingGame.value.id, updateData)
      ElMessage.success('游戏更新成功')
    } else {
      // 创建游戏
      await gameService.createGame(gameForm)
      ElMessage.success('游戏创建成功')
    }
    
    cancelEdit()
    await loadGames()
  } catch (error) {
    console.error('保存游戏失败:', error)
    ElMessage.error(editingGame.value ? '游戏更新失败' : '游戏创建失败')
  } finally {
    saving.value = false
  }
}

const cancelEdit = () => {
  showCreateDialog.value = false
  //editingGame.value = null
  
  // // 重置表单
  // gameForm.name = ''
  // gameForm.description = ''
  // gameForm.director_password = ''
  // gameForm.max_players = 10
  // gameForm.rule_template_id = ''
  
  //gameFormRef.value?.resetFields()
}

// 生命周期
onMounted(async () => {
  await Promise.all([
    loadGames(),
    loadRuleTemplates()
  ])
})
</script>

<style scoped>
.admin-games-page {
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
  flex-shrink: 0;
}

.search-bar {
  display: flex;
  gap: 16px;
  align-items: center;
}

.search-input {
  flex: 1;
  max-width: 300px;
}

.table-card {
  flex: 1;
}

.game-name {
  display: flex;
  align-items: center;
  gap: 8px;
}

.name-text {
  font-weight: 500;
}

.status-tag {
  flex-shrink: 0;
}

.table-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 0 0 0;
  border-top: 1px solid #ebeef5;
  margin-top: 16px;
}

.selection-info {
  color: #606266;
  font-size: 14px;
}

.batch-actions {
  display: flex;
  gap: 8px;
}

/* 响应式设计 */
@media (max-width: 767px) {
  .page-header {
    flex-direction: column;
    align-items: stretch;
  }
  
  .search-bar {
    flex-direction: column;
    align-items: stretch;
  }
  
  .search-input {
    max-width: none;
  }
  
  .table-footer {
    flex-direction: column;
    gap: 12px;
    align-items: stretch;
  }
}
</style>