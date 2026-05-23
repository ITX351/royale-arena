<template>
  <div class="pre-game-management">
    <!-- 游戏属性编辑卡片 -->
    <el-card class="game-properties-card">
      <template #header>
        <div class="card-header">
          <span>游戏属性</span>
          <el-button type="primary" @click="openEditDialog">
            编辑游戏属性
          </el-button>
        </div>
      </template>
      <el-descriptions :column="2" border>
        <el-descriptions-item label="游戏名称">
          {{ game.name }}
        </el-descriptions-item>
        <el-descriptions-item label="游戏描述">
          {{ game.description || '无' }}
        </el-descriptions-item>
      </el-descriptions>
    </el-card>

    <!-- 演员管理 -->
    <ActorManagement 
      :game-id="game.id" 
      :director-password="directorPassword"
      @refresh="$emit('refresh')"
    />
    
    <!-- 规则管理 -->
    <RuleManagement 
      :game="game" 
      :director-password="directorPassword"
      @refresh="$emit('refresh')"
    />

    <!-- 编辑游戏属性对话框 -->
    <el-dialog
      v-model="editDialogVisible"
      title="编辑游戏属性"
      width="500px"
      :close-on-click-modal="false"
    >
      <el-form :model="editForm" label-width="100px">
        <el-form-item label="游戏名称">
          <el-input v-model="editForm.name" maxlength="100" show-word-limit />
        </el-form-item>
        <el-form-item label="游戏描述">
          <el-input
            v-model="editForm.description"
            type="textarea"
            :rows="4"
            placeholder="可选"
          />
        </el-form-item>
      </el-form>
      <template #footer>
        <span class="dialog-footer">
          <el-button @click="editDialogVisible = false">取消</el-button>
          <el-button
            type="primary"
            @click="saveGameProperties"
            :loading="saving"
            :disabled="!hasChanges"
          >
            保存
          </el-button>
        </span>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { ElMessage } from 'element-plus'
import type { GameWithRules } from '@/types/game'
import { directorService } from '@/services/directorService'
import ActorManagement from './ActorManagement.vue'
import RuleManagement from './RuleManagement.vue'

// Props
const props = defineProps<{
  game: GameWithRules
  directorPassword: string
}>()

// Emits
const emit = defineEmits<{
  (e: 'refresh'): void
}>()

// 响应式状态
const editDialogVisible = ref(false)
const saving = ref(false)
const editForm = ref({
  name: '',
  description: ''
})

// 计算属性
const hasChanges = computed(() => {
  return editForm.value.name !== props.game.name ||
         editForm.value.description !== (props.game.description || '')
})

// 方法
const openEditDialog = () => {
  editForm.value = {
    name: props.game.name,
    description: props.game.description || ''
  }
  editDialogVisible.value = true
}

const saveGameProperties = async () => {
  // 验证名称不为空
  if (!editForm.value.name.trim()) {
    ElMessage.error('游戏名称不能为空')
    return
  }

  try {
    saving.value = true
    
    const response = await directorService.editGame(
      props.game.id,
      props.directorPassword,
      {
        name: editForm.value.name,
        description: editForm.value.description || undefined
      }
    )
    
    if (response.success) {
      ElMessage.success('游戏属性保存成功')
      editDialogVisible.value = false
      emit('refresh')
    } else {
      throw new Error(response.error || '保存失败')
    }
  } catch (error: any) {
    console.error('保存游戏属性失败:', error)
    ElMessage.error(error.message || '保存失败，请稍后重试')
  } finally {
    saving.value = false
  }
}
</script>

<style scoped>
.pre-game-management {
  margin-top: 24px;
}

.game-properties-card {
  margin-bottom: 24px;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

@media (max-width: 768px) {
  .pre-game-management {
    margin-top: 16px;
  }
  
  .game-properties-card {
    margin-bottom: 16px;
  }
}
</style>