<template>
  <div class="rule-management">
    <el-card class="rules-card">
      <template #header>
        <div class="card-header">
          <div class="header-left">
            <el-button
              :icon="rulesCollapsed ? ArrowDown : ArrowUp"
              @click="rulesCollapsed = !rulesCollapsed"
              text
              class="collapse-btn"
            >
              游戏规则配置
            </el-button>
          </div>
          <div class="header-actions">
            <el-button
              type="primary"
              @click="saveRules"
              :loading="saving"
              :disabled="!isDirty || rulesCollapsed"
            >
              保存规则
            </el-button>
            <el-button @click="resetRules" :disabled="!isDirty || rulesCollapsed">
              重置
            </el-button>
          </div>
        </div>
      </template>
      
      <el-collapse-transition>
        <div v-show="!rulesCollapsed">
          <div class="rules-content">
            <el-alert
              v-if="!game.rules_config"
              title="当前游戏未配置规则"
              type="info"
              show-icon
              :closable="false"
              class="no-rules-alert"
            />
            
            <div v-else class="rules-editor">
              <el-alert
                title="规则配置说明"
                type="info"
                show-icon
                :closable="false"
                class="rules-info"
              >
                <template #default>
                  <p>您可以在此编辑游戏规则的JSON配置。修改后请点击"保存规则"按钮应用更改。</p>
                  <p>注意：不正确的规则配置可能导致游戏无法正常运行。</p>
                </template>
              </el-alert>
              
              <el-row :gutter="24">
                <el-col :span="24" :md="12">
                  <div class="editor-container">
                    <prism-editor
                      v-model="editableRules"
                      language="json"
                      :highlight="highlighter"
                      line-numbers
                      class="code-editor"
                    />
                  </div>
                </el-col>
                
                <el-col :span="24" :md="12">
                  <div class="rules-preview">
                    <h4>规则预览</h4>
                    <pre class="rules-json">{{ formattedRules }}</pre>
                  </div>
                </el-col>
              </el-row>
            </div>
          </div>
        </div>
      </el-collapse-transition>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { ElMessage } from 'element-plus'
import { ArrowDown, ArrowUp } from '@element-plus/icons-vue'
import { PrismEditor } from 'vue-prism-editor'
import 'vue-prism-editor/dist/prismeditor.min.css'
import { highlight, languages } from 'prismjs'
import 'prismjs/components/prism-json'
import 'prismjs/themes/prism-tomorrow.css'
import type { GameWithRules } from '@/types/game'
import { gameService } from '@/services/gameService'

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
const editableRules = ref('')
const originalRules = ref('')
const saving = ref(false)
const rulesCollapsed = ref(false)

// 计算属性
const isDirty = computed(() => editableRules.value !== originalRules.value)

const formattedRules = computed(() => {
  try {
    const parsed = JSON.parse(editableRules.value)
    return JSON.stringify(parsed, null, 2)
  } catch {
    return editableRules.value
  }
})

// 监听器
watch(() => props.game.rules_config, (newRules) => {
  if (newRules) {
    editableRules.value = JSON.stringify(newRules, null, 2)
    originalRules.value = editableRules.value
  } else {
    editableRules.value = '{}'
    originalRules.value = editableRules.value
  }
}, { immediate: true })

// 方法实现
const highlighter = (code: string) => {
  return highlight(code, languages.json, 'json')
}

const saveRules = async () => {
  if (!isDirty.value) return
  
  try {
    // 验证JSON格式
    const parsedRules = JSON.parse(editableRules.value)
    
    saving.value = true
    
    // 调用管理员接口更新游戏规则
    const response = await gameService.updateGame(props.game.id, {
      rules_config: parsedRules
    })
    
    if (response.success && response.data) {
      ElMessage.success('规则保存成功')
      originalRules.value = editableRules.value
      emit('refresh')
    } else {
      throw new Error(response.message || '保存失败')
    }
  } catch (error: any) {
    console.error('保存规则失败:', error)
    
    if (error instanceof SyntaxError) {
      ElMessage.error('JSON格式错误，请检查配置')
    } else {
      ElMessage.error(error.message || '保存失败，请稍后重试')
    }
  } finally {
    saving.value = false
  }
}

const resetRules = () => {
  editableRules.value = originalRules.value
}
</script>

<style scoped>
.rules-card {
  margin-bottom: 24px;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  flex-wrap: wrap;
  gap: 16px;
}

.card-header h3 {
  margin: 0;
  color: #303133;
}

.header-left {
  display: flex;
  align-items: center;
}

.header-actions {
  display: flex;
  gap: 12px;
}

.collapse-btn {
  font-size: 16px;
  font-weight: 600;
  color: #303133;
}

.rules-content {
  min-height: 300px;
  margin-top: 16px;
}

.no-rules-alert {
  margin-bottom: 20px;
}

.rules-info {
  margin-bottom: 20px;
}

.editor-container {
  margin-bottom: 0;
  border: 1px solid #DCDFE6;
  border-radius: 4px;
  overflow: hidden;
  min-height: 300px;
}

.code-editor {
  min-height: 300px;
  font-size: 14px;
}

.rules-preview {
  margin-top: 0;
}

.rules-preview h4 {
  margin: 0 0 12px 0;
  color: #303133;
}

.rules-json {
  background: #2d2d2d;
  color: #f8f8f2;
  padding: 16px;
  border-radius: 4px;
  overflow-x: auto;
  margin: 0;
  font-size: 13px;
  line-height: 1.5;
}

@media (max-width: 768px) {
  .card-header {
    flex-direction: column;
    align-items: flex-start;
  }
  
  .header-left {
    width: 100%;
  }
  
  .header-actions {
    width: 100%;
    justify-content: flex-end;
  }
  
  .rules-preview {
    margin-top: 20px;
  }
}
</style>
