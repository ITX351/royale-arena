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
            <el-button @click="templateDialogVisible = true" :disabled="rulesCollapsed">
              加载规则模版
            </el-button>
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
              <el-tabs v-model="activeTab" class="rules-tabs">
                <el-tab-pane label="JSON编辑器" name="editor">
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
                      <GameRulesPreview :rules-json="editableRules" class="rules-parser-inline" />
                    </el-col>
                  </el-row>
                </el-tab-pane>
                

                
                <el-tab-pane label="规则文档" name="documentation">
                  <div class="rules-documentation">
                    <el-alert
                      v-if="documentationError"
                      :title="documentationError"
                      type="error"
                      show-icon
                      :closable="false"
                    />
                    <div v-else>
                      <el-tabs v-model="activeDocTab" type="border-card">
                        <el-tab-pane label="配置指南" name="guide">
                          <el-skeleton v-if="loadingDocumentation" :rows="10" animated />
                          <div v-else v-html="renderedDocumentation" class="documentation-content"></div>
                        </el-tab-pane>
                        <el-tab-pane label="使用示例" name="examples">
                          <el-skeleton v-if="loadingExamples" :rows="10" animated />
                          <div v-else v-html="renderedExamples" class="documentation-content"></div>
                        </el-tab-pane>

                      </el-tabs>
                    </div>
                  </div>
                </el-tab-pane>
              </el-tabs>
            </div>
          </div>
        </div>
      </el-collapse-transition>
    </el-card>

    <RuleTemplateDialog
      v-model="templateDialogVisible"
      @select="applyTemplate"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue'
import { ElMessage } from 'element-plus'
import { ArrowDown, ArrowUp } from '@element-plus/icons-vue'
import { PrismEditor } from 'vue-prism-editor'
import 'vue-prism-editor/dist/prismeditor.min.css'
import { highlight, languages } from 'prismjs'
import 'prismjs/components/prism-json'
import 'prismjs/themes/prism-tomorrow.css'
import MarkdownIt from 'markdown-it'

import type { GameWithRules } from '@/types/game'
import type { RuleTemplate } from '@/types/ruleTemplate'
import { directorService } from '@/services/directorService'
import { getBasePathUrl } from '@/utils/commonUtils'
import { GameRuleParser } from '@/utils/gameRuleParser'
import RuleTemplateDialog from '@/views/director/components/RuleTemplateDialog.vue'
import GameRulesPreview from '@/components/GameRulesPreview.vue'

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
const templateDialogVisible = ref(false)
const activeTab = ref('editor')
const activeDocTab = ref('guide')
const documentation = ref('')
const examples = ref('')
const loadingDocumentation = ref(false)
const loadingExamples = ref(false)
const documentationError = ref('')

// 初始化Markdown渲染器
const md = new MarkdownIt({
  html: true,
  linkify: true,
  typographer: true
})

// 计算属性
const isDirty = computed(() => editableRules.value !== originalRules.value)

const renderedDocumentation = computed(() => {
  if (!documentation.value) return ''
  return md.render(documentation.value)
})

const renderedExamples = computed(() => {
  if (!examples.value) return ''
  return md.render(examples.value)
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

// 生命周期钩子
onMounted(() => {
  loadRulesDocumentation()
})

// 方法实现
const highlighter = (code: string) => {
  return highlight(code, languages.json, 'json')
}

const saveRules = async () => {
  if (!isDirty.value) return

  try {
    // 验证JSON格式
    const parsedRules = JSON.parse(editableRules.value)

    // 使用解析器检查是否有解析错误
    const parser = new GameRuleParser();
    const parsedGameRules = parser.parse(parsedRules);

    // 检查是否存在解析错误
    if (parsedGameRules.parsingIssues.length > 0 || parsedGameRules.missingSections.length > 0) {
      // 如果有解析问题或缺失部分，显示错误信息
      const issues = parsedGameRules.parsingIssues.length > 0 
        ? `解析问题：${parsedGameRules.parsingIssues.join('; ')}` 
        : '';
      const missing = parsedGameRules.missingSections.length > 0 
        ? `缺失部分：${parsedGameRules.missingSections.join(', ')}` 
        : '';
      
      const errorMessage = `${issues} ${missing}`.trim();
      ElMessage.error(`存在规则配置错误，请修正后保存：${errorMessage}`);
      return; // 阻止保存
    }

    saving.value = true

    // 调用导演接口更新游戏规则
    const response = await directorService.editGame(props.game.id, props.directorPassword, {
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

const applyTemplate = (template: RuleTemplate) => {
  try {
    const templateConfig =
      template.rules_config && typeof template.rules_config === 'object'
        ? template.rules_config
        : {}
    const formattedRules = JSON.stringify(templateConfig, null, 2)

    editableRules.value = formattedRules
    activeTab.value = 'editor'
    if (rulesCollapsed.value) {
      rulesCollapsed.value = false
    }
    ElMessage.success(`已加载规则模版：${template.template_name}`)
  } catch (error) {
    console.error('应用规则模版失败:', error)
    ElMessage.error('应用规则模版失败')
  }
}

const loadRulesDocumentation = async () => {
  try {
    loadingDocumentation.value = true
    documentationError.value = ''
    
    // 使用统一的函数获取带 base 路径的 URL
    const response = await fetch(getBasePathUrl('docs/game-rules-explain.md'))
    if (!response.ok) {
      throw new Error('无法加载规则文档')
    }
    
    documentation.value = await response.text()
  } catch (error: any) {
    documentationError.value = error.message || '加载规则文档失败'
    console.error('加载规则文档失败:', error)
  } finally {
    loadingDocumentation.value = false
  }
}

const loadRulesExamples = async () => {
  try {
    loadingExamples.value = true
    documentationError.value = ''
    
    // 使用统一的函数获取带 base 路径的 URL
    const response = await fetch(getBasePathUrl('docs/game-rules-examples.md'))
    if (!response.ok) {
      throw new Error('无法加载使用示例')
    }
    
    examples.value = await response.text()
  } catch (error: any) {
    documentationError.value = error.message || '加载使用示例失败'
    console.error('加载使用示例失败:', error)
  } finally {
    loadingExamples.value = false
  }
}

// 监听文档标签页切换
watch(activeDocTab, (newTab) => {
  switch (newTab) {
    case 'guide':
      if (!documentation.value && !loadingDocumentation.value) {
        loadRulesDocumentation()
      }
      break
    case 'examples':
      if (!examples.value && !loadingExamples.value) {
        loadRulesExamples()
      }
      break
  }
})
</script>

<style scoped>
.rule-management {
  max-width: 100%;
  overflow-x: hidden;
  width: 100%;
  display: flex;
  justify-content: center;
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

.rules-card {
  margin-bottom: 24px;
  max-width: 100%;
  overflow-x: hidden;
  width: 100%;
  max-width: 900px;
}

.rules-content {
  min-height: 300px;
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

.rules-documentation-wrapper {
  max-width: 100%;
  overflow-x: hidden;
  width: 100%;
  max-width: 800px;
}

.rules-documentation {
  max-width: 100%;
  overflow-x: hidden;
  width: 100%;
  max-width: 800px;
}

.documentation-content {
  padding: 16px;
  background: #f5f7fa;
  border-radius: 4px;
  line-height: 1.6;
  text-align: left;
  word-break: break-word;
  overflow-wrap: break-word;
  max-width: 100%;
  box-sizing: border-box;
  white-space: normal;
  /* 固定宽度，不随页面宽度变化 */
  width: 100%;
  max-width: 800px; /* Fixed maximum width */
  overflow-x: hidden;
}

.documentation-content h1,
.documentation-content h2,
.documentation-content h3 {
  color: #303133;
  margin-top: 24px;
  margin-bottom: 16px;
}

.documentation-content h1 {
  font-size: 24px;
  border-bottom: 1px solid #dcdfe6;
  padding-bottom: 10px;
}

.documentation-content h2 {
  font-size: 20px;
  border-bottom: 1px solid #dcdfe6;
  padding-bottom: 8px;
}

.documentation-content h3 {
  font-size: 18px;
}

.documentation-content code {
  background: #f0f2f5;
  padding: 2px 4px;
  border-radius: 3px;
  font-family: monospace;
}

.documentation-content pre {
  background: #2d2d2d;
  color: #f8f8f2;
  padding: 16px;
  border-radius: 4px;
  overflow-x: auto;
  white-space: pre-wrap;
  word-wrap: break-word;
}

.documentation-content pre code {
  background: none;
  padding: 0;
  white-space: pre-wrap;
  word-wrap: break-word;
}

/* Mermaid图表样式 */
.mermaid {
  text-align: center;
  background: white;
  padding: 16px;
  border-radius: 4px;
  box-shadow: 0 2px 4px rgba(0,0,0,0.1);
}

.documentation-content blockquote {
  border-left: 4px solid #409eff;
  margin: 16px 0;
  color: #606266;
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