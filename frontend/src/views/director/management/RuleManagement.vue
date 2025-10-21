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
                      <div class="rules-parser-inline">
                        <el-alert
                          v-if="parsedRules.missingSections.length > 0"
                          title="缺失的规则部分"
                          type="warning"
                          show-icon
                          :closable="false"
                          class="parser-warning"
                        >
                          <template #default>
                            <p>以下规则部分缺失或未配置：</p>
                            <ul>
                              <li v-for="section in parsedRules.missingSections" :key="section">
                                {{ section }}
                              </li>
                            </ul>
                          </template>
                        </el-alert>
                        
                        <el-alert
                          v-if="parsedRules.parsingIssues.length > 0"
                          title="解析问题"
                          type="error"
                          show-icon
                          :closable="false"
                          class="parser-error"
                        >
                          <template #default>
                            <ul>
                              <li v-for="issue in parsedRules.parsingIssues" :key="issue">
                                {{ issue }}
                              </li>
                            </ul>
                          </template>
                        </el-alert>
                        
                        <el-collapse v-model="activeParserPanels">
                          <el-collapse-item title="基础规则" name="basic">
                            <div class="parser-section">
                              <el-row :gutter="16">
                                <el-col :span="12">
                                  <h4>地图配置</h4>
                                  <p><strong>地点数量：</strong>{{ parsedRules.map.places.length }}</p>
                                  <p><strong>安全区域：</strong>{{ parsedRules.map.safePlaces.join(', ') || '无' }}</p>
                                  <div>
                                    <strong>地点列表：</strong>
                                    <el-tag v-for="place in parsedRules.map.places" :key="place" style="margin: 2px;">{{ place }}</el-tag>
                                  </div>
                                </el-col>
                                <el-col :span="12">
                                  <h4>玩家配置</h4>
                                  <p><strong>最大生命值：</strong>{{ parsedRules.player.maxLife }}</p>
                                  <p><strong>最大体力值：</strong>{{ parsedRules.player.maxStrength }}</p>
                                  <p><strong>每日生命恢复：</strong>{{ parsedRules.player.dailyLifeRecovery }}</p>
                                  <p><strong>每日体力恢复：</strong>{{ parsedRules.player.dailyStrengthRecovery }}</p>
                                  <p><strong>搜索冷却时间：</strong>{{ parsedRules.player.searchCooldown }}秒</p>
                                  <p><strong>背包最大物品数：</strong>{{ parsedRules.player.maxBackpackItems }}</p>
                                </el-col>
                              </el-row>
                              
                              <el-row :gutter="16">
                                <el-col :span="12">
                                  <h4>行动消耗</h4>
                                  <p><strong>移动：</strong>{{ parsedRules.actionCosts.move }}体力</p>
                                  <p><strong>搜索：</strong>{{ parsedRules.actionCosts.search }}体力</p>
                                  <p><strong>拾取：</strong>{{ parsedRules.actionCosts.pick }}体力</p>
                                  <p><strong>攻击：</strong>{{ parsedRules.actionCosts.attack }}体力</p>
                                </el-col>
                                <el-col :span="12">
                                  <h4>静养模式</h4>
                                  <p><strong>生命恢复：</strong>{{ parsedRules.restMode.lifeRecovery }}点</p>
                                  <p><strong>体力恢复：</strong>{{ parsedRules.restMode.strengthRecovery }}点</p>
                                  <p><strong>最大移动次数：</strong>{{ parsedRules.restMode.maxMoves }}次</p>
                                  <p><strong>队友行为规则：</strong>{{ parsedRules.teammateBehavior }}</p>
                                  <p><strong>死亡后物品去向：</strong>{{ getDispositionDisplayText(parsedRules.deathItemDisposition) }}</p>
                                  
                                  <!-- 队友行为详细解析 -->
                                  <div class="teammate-behavior-details">
                                    <h5>队友行为详细设置：</h5>
                                    <el-tag type="info" v-if="parsedRules.parsedTeammateBehaviors.noHarm" style="margin: 2px;">禁止队友伤害</el-tag>
                                    <el-tag type="info" v-if="parsedRules.parsedTeammateBehaviors.noSearch" style="margin: 2px;">禁止搜索到队友</el-tag>
                                    <el-tag type="info" v-if="parsedRules.parsedTeammateBehaviors.canViewStatus" style="margin: 2px;">允许查看队友状态</el-tag>
                                    <el-tag type="info" v-if="parsedRules.parsedTeammateBehaviors.canTransferItems" style="margin: 2px;">允许赠送物品给队友</el-tag>
                                    
                                    <!-- 如果没有启用任何特殊行为，显示默认 -->
                                    <el-tag v-if="!parsedRules.parsedTeammateBehaviors.noHarm && 
                                               !parsedRules.parsedTeammateBehaviors.noSearch && 
                                               !parsedRules.parsedTeammateBehaviors.canViewStatus && 
                                               !parsedRules.parsedTeammateBehaviors.canTransferItems" 
                                            style="margin: 2px;" 
                                            type="success">
                                      无特殊队友行为规则
                                    </el-tag>
                                  </div>
                                </el-col>
                              </el-row>
                            </div>
                          </el-collapse-item>
                          
                          <el-collapse-item title="物品系统" name="items">
                            <div class="parser-section">
                              <el-tabs v-model="activeItemTab" type="card">
                                <el-tab-pane label="稀有度级别" name="rarity">
                                  <el-table :data="parsedRules.itemsConfig.rarityLevels" style="width: 100%">
                                    <el-table-column prop="internalName" label="内部名称" />
                                    <el-table-column prop="displayName" label="显示名称" />
                                    <el-table-column prop="prefix" label="前缀" />
                                    <el-table-column label="是否空投">
                                      <template #default="scope">
                                        {{ scope.row.isAirdropped ? '是' : '否' }}
                                      </template>
                                    </el-table-column>
                                  </el-table>
                                </el-tab-pane>
                                
                                <el-tab-pane label="武器" name="weapons">
                                  <el-table :data="parsedRules.itemsConfig.items.weapons" style="width: 100%">
                                    <el-table-column prop="internalName" label="内部名称" />
                                    <el-table-column prop="rarity" label="稀有度" />
                                    <el-table-column label="显示名称">
                                      <template #default="scope">
                                        {{ scope.row.displayNames.join(', ') }}
                                      </template>
                                    </el-table-column>
                                    <el-table-column label="属性">
                                      <template #default="scope">
                                        <div>伤害: {{ scope.row.properties.damage }}</div>
                                        <div v-if="scope.row.properties.uses !== undefined">使用次数: {{ scope.row.properties.uses }}</div>
                                        <div>票数: {{ scope.row.properties.votes }}</div>
                                        <div v-if="scope.row.properties.aoeDamage !== undefined">范围伤害: {{ scope.row.properties.aoeDamage }}</div>
                                        <div v-if="scope.row.properties.bleedDamage !== undefined">流血伤害: {{ scope.row.properties.bleedDamage }}</div>
                                      </template>
                                    </el-table-column>
                                  </el-table>
                                </el-tab-pane>

                                <el-tab-pane label="防具" name="armors">
                                  <el-table :data="parsedRules.itemsConfig.items.armors" style="width: 100%">
                                    <el-table-column prop="internalName" label="内部名称" />
                                    <el-table-column prop="rarity" label="稀有度" />
                                    <el-table-column label="显示名称">
                                      <template #default="scope">
                                        {{ scope.row.displayNames.join(', ') }}
                                      </template>
                                    </el-table-column>
                                    <el-table-column label="属性">
                                      <template #default="scope">
                                        <div>防御: {{ scope.row.properties.defense }}</div>
                                        <div>票数: {{ scope.row.properties.votes }}</div>
                                        <div v-if="scope.row.properties.uses !== undefined">使用次数: {{ scope.row.properties.uses }}</div>
                                      </template>
                                    </el-table-column>
                                  </el-table>
                                </el-tab-pane>
                                
                                <el-tab-pane label="功能物品" name="utilities">
                                  <el-table :data="parsedRules.itemsConfig.items.utilities" style="width: 100%">
                                    <el-table-column prop="name" label="名称" />
                                    <el-table-column prop="internalName" label="内部名称" />
                                    <el-table-column prop="rarity" label="稀有度" />
                                    <el-table-column label="类别">
                                      <template #default="scope">
                                        {{ scope.row.properties.category || '未指定' }}
                                      </template>
                                    </el-table-column>
                                    <el-table-column label="属性">
                                      <template #default="scope">
                                        <div>
                                          <div v-if="scope.row.properties.uses !== undefined">使用次数: {{ scope.row.properties.uses }}</div>
                                          <div v-if="scope.row.properties.usesNight !== undefined">每晚使用次数: {{ scope.row.properties.usesNight }}</div>
                                          <div v-if="scope.row.properties.votes !== undefined">票数: {{ scope.row.properties.votes }}</div>
                                          <div v-if="scope.row.properties.targets !== undefined">目标数: {{ scope.row.properties.targets }}</div>
                                          <div v-if="scope.row.properties.damage !== undefined">伤害: {{ scope.row.properties.damage }}</div>
                                        </div>
                                      </template>
                                    </el-table-column>
                                  </el-table>
                                </el-tab-pane>
                                
                                <el-tab-pane label="升级道具" name="upgraders">
                                  <el-table :data="parsedRules.itemsConfig.items.upgraders" style="width: 100%">
                                    <el-table-column prop="internalName" label="内部名称" />
                                    <el-table-column prop="rarity" label="稀有度" />
                                    <el-table-column label="显示名称">
                                      <template #default="scope">
                                        {{ scope.row.displayNames.join(', ') }}
                                      </template>
                                    </el-table-column>
                                  </el-table>
                                </el-tab-pane>
                                
                                <el-tab-pane label="合成配方" name="recipes">
                                  <div v-for="(recipes, upgrader) in parsedRules.itemsConfig.upgradeRecipes" :key="upgrader">
                                    <h4>{{ upgrader }}</h4>
                                    <el-table :data="recipes" style="width: 100%">
                                      <el-table-column prop="result" label="结果" />
                                      <el-table-column label="材料">
                                        <template #default="scope">
                                          {{ scope.row.ingredients.join(', ') }}
                                        </template>
                                      </el-table-column>
                                    </el-table>
                                  </div>
                                </el-tab-pane>
                                <el-tab-pane label="消耗品" name="consumables">
                                  <el-table :data="parsedRules.itemsConfig.items.consumables" style="width: 100%">
                                    <el-table-column prop="name" label="名称" />
                                    <el-table-column label="效果类型">
                                      <template #default="scope">
                                        {{ scope.row.properties.effectType }}
                                      </template>
                                    </el-table-column>
                                    <el-table-column label="效果值">
                                      <template #default="scope">
                                        {{ scope.row.properties.effectValue }}
                                      </template>
                                    </el-table-column>
                                    <el-table-column label="治愈流血">
                                      <template #default="scope">
                                        <span v-if="scope.row.properties.cureBleed === undefined">否</span>
                                        <span v-else-if="scope.row.properties.cureBleed === 1">抵消</span>
                                        <span v-else>治愈</span>
                                      </template>
                                    </el-table-column>
                                  </el-table>
                                </el-tab-pane>
                              </el-tabs>
                            </div>
                          </el-collapse-item>
                        </el-collapse>
                      </div>
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
import { GameRuleParser, type ParsedGameRules } from '@/utils/gameRuleParser'
import RuleTemplateDialog from '@/views/director/components/RuleTemplateDialog.vue'

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
const activeParserPanels = ref(['basic', 'items'])
const activeItemTab = ref('rarity')
const activeDocTab = ref('guide')
const documentation = ref('')
const examples = ref('')
const loadingDocumentation = ref(false)
const loadingExamples = ref(false)
const documentationError = ref('')

// 解析器实例
const ruleParser = new GameRuleParser()

// 初始化Markdown渲染器
const md = new MarkdownIt({
  html: true,
  linkify: true,
  typographer: true
})



// 计算属性
const isDirty = computed(() => editableRules.value !== originalRules.value)

const parsedRules = computed<ParsedGameRules>(() => {
  try {
    const rules = JSON.parse(editableRules.value)
    return ruleParser.parse(rules)
  } catch {
    return ruleParser.parse({})
  }
})



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

const getDispositionDisplayText = (value: string) => {
  const dispositionMap: Record<string, string> = {
    'killer_takes_loot': '由击杀者收缴（无击杀者则掉落在原地）',
    'drop_to_ground': '无条件掉落在原地',
    'vanish_completely': '凭空消失'
  };
  return dispositionMap[value] || value;
}

const loadRulesDocumentation = async () => {
  try {
    loadingDocumentation.value = true
    documentationError.value = ''
    
    const response = await fetch('/docs/game-rules-explain.md')
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
    
    const response = await fetch('/docs/game-rules-examples.md')
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
  margin-top: 16px;
}

.no-rules-alert {
  margin-bottom: 20px;
}

.rules-info {
  margin-bottom: 20px;
}

.rules-tabs {
  margin-top: 20px;
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

.rules-parser {
  padding: 20px 0;
}

.parser-section {
  padding: 16px;
}

.parser-section h4 {
  margin-top: 0;
  color: #303133;
}

.parser-warning,
.parser-error {
  margin-bottom: 20px;
}

.rules-documentation-wrapper {
  max-width: 100%;
  overflow-x: hidden;
  width: 100%;
  max-width: 800px;
}

.rules-documentation {
  padding: 20px 0;
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
  margin: 16px 0;
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
  margin: 16px 0;
  background: white;
  padding: 16px;
  border-radius: 4px;
  box-shadow: 0 2px 4px rgba(0,0,0,0.1);
}

.documentation-content blockquote {
  border-left: 4px solid #409eff;
  padding: 0 16px;
  margin: 16px 0;
  color: #606266;
}

.teammate-behavior-details {
  margin-top: 12px;
  padding: 8px;
  background-color: #f5f7fa;
  border-radius: 4px;
}

.teammate-behavior-details h5 {
  margin-top: 0;
  margin-bottom: 8px;
  color: #606266;
  font-size: 14px;
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