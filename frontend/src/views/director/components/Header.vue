<template>
  <div class="director-header">
    <el-card class="header-card" :class="{ 'hide-body': !showDetails }">
      <template #header>
        <div class="card-header">
          <div class="header-title">
            <h2>{{ game.name }}</h2>
            <el-button 
              type="primary" 
              link 
              @click="showDetails = !showDetails"
              :icon="showDetails ? ArrowUp : ArrowDown"
              class="toggle-button"
            />
          </div>
          <div class="header-actions">
            <el-button 
              v-if="showStartButton"
              type="primary"
              @click="confirmStartGame"
              :loading="actionLoading"
            >
              开始游戏
            </el-button>
            <el-button 
              v-if="showPauseButton"
              type="warning"
              @click="pauseGame"
              :loading="actionLoading"
            >
              暂停游戏
            </el-button>
            <el-button 
              v-if="showSaveButton"
              type="primary"
              @click="saveGame"
              :loading="actionLoading"
            >
              存盘游戏
            </el-button>
            <el-button 
              v-if="showResumeButton"
              type="success"
              @click="resumeGame"
              :loading="actionLoading"
            >
              继续游戏
            </el-button>
            <el-button 
              v-if="showEndButton"
              type="danger"
              @click="confirmEndGame"
              :loading="actionLoading"
            >
              结束游戏
            </el-button>
            <!-- <el-button @click="goBack">
              返回游戏详情
            </el-button> -->
            <el-button @click="goHome">
              返回首页
            </el-button>
          </div>
        </div>
        <div class="game-status-line">
          <el-tag :type="statusTagType" size="large">
            {{ statusDisplayText }}
          </el-tag>
          <span>演员人数: {{ game.player_count }}</span>
        </div>
      </template>
      
      <div v-show="showDetails" class="game-details">
        <p v-if="game.description">{{ game.description }}</p>
        <p v-else class="no-description">暂无游戏描述</p>
        <div class="game-stats">
          <span>创建时间: {{ formatDate(game.created_at) }}</span>
          <span class="game-id">游戏ID: {{ game.id }}</span>
        </div>
      </div>
    </el-card>
    
    <!-- 存档文件选择对话框 -->
    <el-dialog
      v-model="showSaveFileDialog"
      title="选择存档文件"
      width="500px"
      :before-close="handleSaveFileClose"
    >
      <el-table 
        :data="saveFiles" 
        highlight-current-row
        @current-change="handleSaveFileSelect"
        style="width: 100%"
      >
        <el-table-column prop="file_name" label="文件名" />
        <el-table-column prop="created_at" label="创建时间" :formatter="formatSaveFileTime" />
      </el-table>
      <template #footer>
        <span class="dialog-footer">
          <el-button @click="showSaveFileDialog = false">取消</el-button>
          <el-button 
            type="primary" 
            @click="confirmResumeGame"
            :disabled="!selectedSaveFile"
          >
            确认恢复
          </el-button>
        </span>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRouter } from 'vue-router'
import { ElMessage, ElMessageBox } from 'element-plus'
import { ArrowDown, ArrowUp } from '@element-plus/icons-vue'
import type { GameWithRules } from '@/types/game'
import { GameStatus } from '@/types/game'
import { directorService } from '@/services/directorService'
import { 
  formatGameStatus, 
  getStatusTagType,
  shouldShowStartButton,
  shouldShowPauseButton,
  shouldShowResumeButton,
  shouldShowEndButton
} from '@/utils/gameUtils'

// Props
const props = defineProps<{
  game: GameWithRules
  directorPassword: string
}>()

// Emits
const emit = defineEmits<{
  (e: 'status-updated'): void
}>()

// Router
const router = useRouter()

// 响应式状态
const actionLoading = ref(false)
const showDetails = ref(false)
const showSaveFileDialog = ref(false)
const saveFiles = ref<any[]>([])
const selectedSaveFile = ref<any>(null)

// 计算属性
const statusDisplayText = computed(() => {
  return formatGameStatus(props.game.status)
})

const statusTagType = computed(() => {
  return getStatusTagType(props.game.status)
})

const showStartButton = computed(() => shouldShowStartButton(props.game.status))
const showPauseButton = computed(() => shouldShowPauseButton(props.game.status))
const showSaveButton = computed(() => {
  // 存盘按钮在游戏状态为 RUNNING 或 PAUSED 时显示
  return props.game.status === GameStatus.RUNNING
})
const showResumeButton = computed(() => shouldShowResumeButton(props.game.status))
const showEndButton = computed(() => shouldShowEndButton(props.game.status))

// 方法实现
const formatDate = (dateString: string) => {
  return new Date(dateString).toLocaleString('zh-CN')
}

const formatSaveFileTime = (row: any) => {
  if (row.created_at) {
    return new Date(row.created_at).toLocaleString('zh-CN')
  }
  return '未知时间'
}

const updateGameStatus = async (targetStatus: GameStatus, saveFileName?: string) => {
  if (!props.directorPassword) {
    ElMessage.error('缺少导演密码')
    return
  }
  
  actionLoading.value = true
  
  try {
    const response = await directorService.updateGameStatus(
      props.game.id,
      props.directorPassword,
      targetStatus,
      saveFileName
    )
    
    if (response.success) {
      ElMessage.success('游戏状态更新成功')
      emit('status-updated')
    } else {
      throw new Error(response.message || '状态更新失败')
    }
  } catch (error: any) {
    console.error('更新游戏状态失败:', error)
    ElMessage.error(
      error.response?.status === 401 
        ? '导演密码错误' 
        : (error.message || '状态更新失败')
    )
  } finally {
    actionLoading.value = false
  }
}

const confirmStartGame = () => {
  ElMessageBox.confirm(
    '确定要开始游戏吗？开始后无法回到等待状态。',
    '确认开始游戏',
    {
      confirmButtonText: '确定',
      cancelButtonText: '取消',
      type: 'warning'
    }
  ).then(() => {
    updateGameStatus(GameStatus.RUNNING)
  }).catch(() => {
    // 用户取消操作
  })
}

const pauseGame = () => {
  ElMessageBox.confirm(
    '确定要暂停游戏吗？暂停游戏会自动执行一次存盘。',
    '暂停游戏',
    {
      confirmButtonText: '确定',
      cancelButtonText: '取消',
      type: 'warning',
    }
  ).then(async () => {
    // 更新游戏状态为暂停
    await updateGameStatus(GameStatus.PAUSED)
    // 发送暂停游戏指令
    // 注意：这里我们不直接断开WebSocket连接，而是在DirectorMain中监听状态变化
    ElMessage.success('游戏已暂停')
  }).catch(() => {
    // 用户取消操作
  })
}

const saveGame = async () => {
  if (!props.directorPassword) {
    ElMessage.error('缺少导演密码')
    return
  }
  
  actionLoading.value = true
  
  try {
    const response = await directorService.manualSaveGame(
      props.game.id,
      props.directorPassword
    )
    
    if (response.success) {
      ElMessage.success(`游戏状态已保存至: ${response.save_file_name}`)
      emit('status-updated')
    } else {
      throw new Error(response.message || '存盘失败')
    }
  } catch (error: any) {
    console.error('存盘失败:', error)
    ElMessage.error(
      error.response?.status === 401 
        ? '导演密码错误' 
        : (error.message || '存盘失败')
    )
  } finally {
    actionLoading.value = false
  }
}

const resumeGame = async () => {
  if (!props.directorPassword) {
    ElMessage.error('缺少导演密码')
    return
  }
  
  try {
    // 获取存档文件列表
    const response = await directorService.listSaveFiles(
      props.game.id,
      props.directorPassword
    )
    
    if (response.success) {
      saveFiles.value = response.data
      selectedSaveFile.value = null
      showSaveFileDialog.value = true
    } else {
      throw new Error(response.message || '获取存档列表失败')
    }
  } catch (error: any) {
    console.error('获取存档列表失败:', error)
    ElMessage.error(
      error.response?.status === 401 
        ? '导演密码错误' 
        : (error.message || '获取存档列表失败')
    )
  }
}

const handleSaveFileSelect = (row: any) => {
  selectedSaveFile.value = row
}

const handleSaveFileClose = () => {
  showSaveFileDialog.value = false
  selectedSaveFile.value = null
}

const confirmResumeGame = async () => {
  if (!selectedSaveFile.value) {
    ElMessage.warning('请选择一个存档文件')
    return
  }
  
  showSaveFileDialog.value = false
  await updateGameStatus(GameStatus.RUNNING, selectedSaveFile.value.file_name)
  selectedSaveFile.value = null
}

const confirmEndGame = () => {
  ElMessageBox.confirm(
    '确定要结束游戏吗？结束后无法重新开始。',
    '确认结束游戏',
    {
      confirmButtonText: '确定',
      cancelButtonText: '取消',
      type: 'warning'
    }
  ).then(() => {
    updateGameStatus(GameStatus.ENDED)
    ElMessage.success('游戏已结束')
  }).catch(() => {
    // 用户取消操作
  })
}

const goBack = () => {
  router.push(`/game/${props.game.id}`)
}

const goHome = () => {
  router.push('/')
}
</script>

<style scoped>
@import '@/styles/shared-header.css';

.director-header {
  margin-bottom: 24px;
}

.header-actions {
  display: flex;
  gap: 12px;
  flex-wrap: wrap;
  flex-shrink: 0;
}
</style>