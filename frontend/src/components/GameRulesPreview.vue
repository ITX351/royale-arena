<template>
  <div class="game-rules-preview">
    <el-alert
      v-if="parseError"
      :title="parseError"
      type="error"
      show-icon
      :closable="false"
      class="parse-error"
    />

    <el-empty
      v-if="!hasConfig && !parseError"
      description="暂无规则配置"
      class="empty-placeholder"
    />

    <template v-else>
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

      <el-collapse v-model="activePanels">
        <el-collapse-item title="基础规则" name="basic">
          <div class="parser-section">
            <el-row :gutter="16">
              <el-col :xs="24" :md="12">
                <h4>地图配置</h4>
                <p><strong>地点数量：</strong>{{ parsedRules.map.places.length }}</p>
                <p><strong>安全区域：</strong>{{ parsedRules.map.safePlaces.join(', ') || '无' }}</p>
                <div>
                  <strong>地点列表：</strong>
                  <el-tag v-for="place in parsedRules.map.places" :key="place" class="tag">{{ place }}</el-tag>
                </div>
              </el-col>
              <el-col :xs="24" :md="12">
                <h4>玩家配置</h4>
                <p><strong>最大生命值：</strong>{{ parsedRules.player.maxLife }}</p>
                <p><strong>最大体力值：</strong>{{ parsedRules.player.maxStrength }}</p>
                <p><strong>每日生命恢复：</strong>{{ parsedRules.player.dailyLifeRecovery }}</p>
                <p><strong>每日体力恢复：</strong>{{ parsedRules.player.dailyStrengthRecovery }}</p>
                <p><strong>搜索冷却时间：</strong>{{ parsedRules.player.searchCooldown }}秒</p>
                <p><strong>背包最大物品数：</strong>{{ parsedRules.player.maxBackpackItems }}</p>
                <p><strong>挥拳伤害：</strong>{{ parsedRules.player.unarmedDamage }}</p>
              </el-col>
            </el-row>

            <el-row :gutter="16">
              <el-col :xs="24" :md="12">
                <h4>行动消耗</h4>
                <p><strong>移动：</strong>{{ parsedRules.actionCosts.move }}体力</p>
                <p><strong>搜索：</strong>{{ parsedRules.actionCosts.search }}体力</p>
                <p><strong>拾取：</strong>{{ parsedRules.actionCosts.pick }}体力</p>
                <p><strong>攻击：</strong>{{ parsedRules.actionCosts.attack }}体力</p>
                <p><strong>装备：</strong>{{ parsedRules.actionCosts.equip }}体力</p>
                <p><strong>使用：</strong>{{ parsedRules.actionCosts.use }}体力</p>
                <p><strong>丢弃：</strong>{{ parsedRules.actionCosts.throw }}体力</p>
                <p><strong>传音：</strong>{{ parsedRules.actionCosts.deliver }}体力</p>
              </el-col>
              <el-col :xs="24" :md="12">
                <h4>静养模式</h4>
                <p><strong>生命恢复：</strong>{{ parsedRules.restMode.lifeRecovery }}点</p>
                <p><strong>体力恢复：</strong>{{ parsedRules.restMode.strengthRecovery }}点</p>
                <p><strong>最大移动次数：</strong>{{ parsedRules.restMode.maxMoves }}次</p>
                <p><strong>队友行为规则：</strong>{{ parsedRules.teammateBehavior }}</p>
                <p><strong>死亡后物品去向：</strong>{{ getDispositionDisplayText(parsedRules.deathItemDisposition) }}</p>

                <div class="teammate-behavior-details">
                  <h5>队友行为详细设置：</h5>
                  <el-tag type="info" v-if="parsedRules.parsedTeammateBehaviors.noHarm" class="tag">禁止队友伤害</el-tag>
                  <el-tag type="info" v-if="parsedRules.parsedTeammateBehaviors.noSearch" class="tag">禁止搜索到队友</el-tag>
                  <el-tag type="info" v-if="parsedRules.parsedTeammateBehaviors.canViewStatus" class="tag">允许查看队友状态</el-tag>
                  <el-tag type="info" v-if="parsedRules.parsedTeammateBehaviors.canTransferItems" class="tag">允许赠送物品给队友</el-tag>
                  <el-tag
                    v-if="!parsedRules.parsedTeammateBehaviors.noHarm &&
                       !parsedRules.parsedTeammateBehaviors.noSearch &&
                       !parsedRules.parsedTeammateBehaviors.canViewStatus &&
                       !parsedRules.parsedTeammateBehaviors.canTransferItems"
                    type="success"
                    class="tag"
                  >
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
                <div class="table-wrapper">
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
                </div>
              </el-tab-pane>

              <el-tab-pane label="武器" name="weapons">
                <div class="table-wrapper">
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
                </div>
              </el-tab-pane>

              <el-tab-pane label="防具" name="armors">
                <div class="table-wrapper">
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
                </div>
              </el-tab-pane>

              <el-tab-pane label="功能物品" name="utilities">
                <div class="table-wrapper">
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
                        <div v-if="scope.row.properties.uses !== undefined">使用次数: {{ scope.row.properties.uses }}</div>
                        <div v-if="scope.row.properties.uses_night !== undefined">每晚使用次数: {{ scope.row.properties.uses_night }}</div>
                        <div v-if="scope.row.properties.votes !== undefined">票数: {{ scope.row.properties.votes }}</div>
                        <div v-if="scope.row.properties.targets !== undefined">目标数: {{ scope.row.properties.targets }}</div>
                        <div v-if="scope.row.properties.damage !== undefined">伤害: {{ scope.row.properties.damage }}
                        </div>
                      </template>
                    </el-table-column>
                  </el-table>
                </div>
              </el-tab-pane>

              <el-tab-pane label="升级道具" name="upgraders">
                <div class="table-wrapper">
                  <el-table :data="parsedRules.itemsConfig.items.upgraders" style="width: 100%">
                    <el-table-column prop="internalName" label="内部名称" />
                    <el-table-column prop="rarity" label="稀有度" />
                    <el-table-column label="显示名称">
                      <template #default="scope">
                        {{ scope.row.displayNames.join(', ') }}
                      </template>
                    </el-table-column>
                  </el-table>
                </div>
              </el-tab-pane>

              <el-tab-pane label="合成配方" name="recipes">
                <div v-for="(recipes, upgrader) in parsedRules.itemsConfig.upgradeRecipes" :key="upgrader" class="recipe-block">
                  <h4>{{ upgrader }}</h4>
                  <div class="table-wrapper">
                    <el-table :data="recipes" style="width: 100%">
                      <el-table-column prop="result" label="结果" />
                      <el-table-column label="材料">
                        <template #default="scope">
                          {{ scope.row.ingredients.join(', ') }}
                        </template>
                      </el-table-column>
                    </el-table>
                  </div>
                </div>
              </el-tab-pane>

              <el-tab-pane label="消耗品" name="consumables">
                <div class="table-wrapper">
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
                </div>
              </el-tab-pane>
            </el-tabs>
          </div>
        </el-collapse-item>
      </el-collapse>
    </template>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { GameRuleParser, type ParsedGameRules } from '@/utils/gameRuleParser'

interface ParsedInputState {
  config: Record<string, unknown>
  hasConfig: boolean
  error: string | null
}

const props = withDefaults(defineProps<{
  rulesJson?: string | null
  rulesConfig?: Record<string, unknown> | null
}>(), {
  rulesJson: null,
  rulesConfig: null
})

const parser = new GameRuleParser()

const activePanels = ref<string[]>(['basic', 'items'])
const activeItemTab = ref('rarity')

const parseState = computed<ParsedInputState>(() => {
  if (props.rulesConfig) {
    return {
      config: props.rulesConfig,
      hasConfig: true,
      error: null
    }
  }

  if (props.rulesJson && props.rulesJson.trim().length > 0) {
    try {
      return {
        config: JSON.parse(props.rulesJson),
        hasConfig: true,
        error: null
      }
    } catch (error) {
      console.error('Failed to parse rules JSON:', error)
      return {
        config: {},
        hasConfig: false,
        error: 'JSON格式错误，已使用空配置展示默认解析结果'
      }
    }
  }

  return {
    config: {},
    hasConfig: false,
    error: null
  }
})

const parsedRules = computed<ParsedGameRules>(() => parser.parse(parseState.value.config))
const hasConfig = computed(() => parseState.value.hasConfig)
const parseError = computed(() => parseState.value.error)

const getDispositionDisplayText = (value: string) => {
  const dispositionMap: Record<string, string> = {
    killer_takes_loot: '由击杀者收缴（无击杀者则掉落在原地）',
    drop_to_ground: '无条件掉落在原地',
    vanish_completely: '凭空消失'
  }
  return dispositionMap[value] || value
}
</script>

<style scoped>
.game-rules-preview {
  width: 100%;
}

.parser-warning,
.parser-error,
.parse-error {
  margin-bottom: 20px;
}

.parser-section {
  padding: 16px;
}

.table-wrapper {
  width: 100%;
  overflow-x: auto;
}

.tag {
  margin: 2px;
}

.teammate-behavior-details {
  margin-top: 12px;
  padding: 8px;
  background-color: #f5f7fa;
  border-radius: 4px;
}

.teammate-behavior-details h5 {
  margin: 0 0 8px 0;
  color: #606266;
  font-size: 14px;
}

.recipe-block {
  margin-bottom: 16px;
}

.empty-placeholder {
  margin: 24px 0;
}

@media (max-width: 768px) {
  .parser-section {
    padding: 12px;
  }

  .game-rules-preview {
    padding-bottom: 8px;
  }

  .table-wrapper {
    margin-bottom: 12px;
  }
}
</style>
