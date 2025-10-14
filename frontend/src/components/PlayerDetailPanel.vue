<template>
  <div class="player-detail-panel">
    <h3>{{ player.name }} 详细状态</h3>
    
    <!-- 基础状态 -->
    <section class="detail-section">
      <h4>基础状态</h4>
      <div class="status-grid">
        <div class="status-item">
          <label>位置:</label>
          <span>{{ player.location || '未出生' }}</span>
        </div>
        <div class="status-item">
          <label>队伍:</label>
          <span>{{ player.team_id ? `队伍${player.team_id}` : '无' }}</span>
        </div>
        <div class="status-item">
          <label>状态:</label>
          <span :class="statusClass">{{ statusText }}</span>
        </div>
      </div>
    </section>
    
    <!-- 生命体力详情 -->
    <section class="detail-section">
      <h4>生命体力</h4>
      <div class="health-detail">
        <div class="health-item">
          <label>生命值:</label>
          <div class="value-with-bar">
            <span>{{ player.life }}/{{ player.max_life }}</span>
            <div class="progress-bar">
              <div 
                class="progress-fill life" 
                :style="{ width: lifePercentage + '%' }"
              ></div>
            </div>
          </div>
        </div>
        <div class="health-item">
          <label>体力值:</label>
          <div class="value-with-bar">
            <span>{{ player.strength }}/{{ player.max_strength }}</span>
            <div class="progress-bar">
              <div 
                class="progress-fill strength" 
                :style="{ width: strengthPercentage + '%' }"
              ></div>
            </div>
          </div>
        </div>
      </div>
    </section>
    
    <!-- 持续效果 -->
    <section v-if="player.bleed_rounds_remaining > 0" class="detail-section">
      <h4>持续效果</h4>
      <div class="effect-item bleed-effect">
        <span class="effect-icon">🩸</span>
        <span class="effect-text">
          流血效果: 每回合失去{{ player.bleed_damage }}生命值，剩余{{ player.bleed_rounds_remaining }}回合
        </span>
      </div>
    </section>
    
    <!-- 装备详情 -->
    <section class="detail-section">
      <h4>装备详情</h4>
      <div class="equipment-detail">
        <div class="equipment-category">
          <h5>武器</h5>
          <div class="equipment-list">
            <ItemDisplay v-if="player.equipped_weapon" :item="player.equipped_weapon" />
            <span v-else class="no-item">未装备</span>
          </div>
        </div>
        
        <div class="equipment-category">
          <h5>防具</h5>
          <div class="equipment-list">
            <ItemDisplay v-if="player.equipped_armor" :item="player.equipped_armor" />
            <span v-else class="no-item">未装备</span>
          </div>
        </div>
      </div>
    </section>
    
    <!-- 背包详情 -->
    <section class="detail-section">
      <h4>背包物品 ({{ player.inventory.length }}/{{ maxBackpack }})</h4>
      <div class="inventory-grid">
        <ItemDisplay 
          v-for="item in player.inventory" 
          :key="item.id"
          :item="item"
          class="inventory-item"
        />
      </div>
    </section>
    
    <!-- 最近活动 -->
    <section class="detail-section">
      <h4>最近活动</h4>
      <div class="activity-info">
        <div class="activity-item">
          <label>上次搜索:</label>
          <span>{{ formatSearchTime(player.last_search_time) }}</span>
        </div>
        <div class="activity-item">
          <label>搜索结果:</label>
          <span>{{ formatSearchResult(player.last_search_result) }}</span>
        </div>
        <div class="activity-item">
          <label>持有票数:</label>
          <span>{{ player.votes }}</span>
        </div>
      </div>
    </section>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { Player, Item } from '@/types/gameStateTypes'
import ItemDisplay from './ItemDisplay.vue'

interface Props {
  player: Player
  maxWeapons?: number
  maxArmors?: number
  maxBackpack?: number
}

const props = withDefaults(defineProps<Props>(), {
  maxWeapons: 1,
  maxArmors: 1,
  maxBackpack: 4
})

const lifePercentage = computed(() => {
  return props.player.max_life > 0 ? (props.player.life / props.player.max_life) * 100 : 0
})

const strengthPercentage = computed(() => {
  return props.player.max_strength > 0 ? (props.player.strength / props.player.max_strength) * 100 : 0
})

const statusClass = computed(() => {
  if (!props.player.is_alive) return 'status-dead'
  if (props.player.is_bound) return 'status-bound'
  if (props.player.rest_mode) return 'status-rest'
  return 'status-normal'
})

const statusText = computed(() => {
  if (!props.player.is_alive) return '已死亡'
  if (props.player.is_bound) return '被捆绑'
  if (props.player.rest_mode) return '静养模式'
  return '正常'
})

const formatSearchTime = (timestamp: string | null): string => {
  if (!timestamp) return '无'
  const date = new Date(timestamp)
  return date.toLocaleTimeString()
}

const formatSearchResult = (result: any): string => {
  if (!result) return '无'
  const visibility = result.is_visible ? '可见' : '不可见'
  const type = result.target_type === 'player' ? '玩家' : '物品'
  return `${type}: ${result.target_name} (${visibility})`
}
</script>

<style scoped>
.player-detail-panel {
  padding: 16px;
  background: white;
  border-radius: 8px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.detail-section {
  margin-bottom: 24px;
  padding-bottom: 16px;
  border-bottom: 1px solid #eee;
}

.detail-section:last-child {
  border-bottom: none;
  margin-bottom: 0;
}

h3 {
  margin: 0 0 16px 0;
  color: #333;
  font-size: 18px;
}

h4 {
  margin: 0 0 12px 0;
  color: #555;
  font-size: 16px;
}

h5 {
  margin: 0 0 8px 0;
  color: #666;
  font-size: 14px;
}

.status-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 12px;
}

.status-item {
  display: flex;
  align-items: center;
  gap: 8px;
}

.status-item label {
  font-weight: 500;
  min-width: 60px;
}

.status-dead { color: #f56565; }
.status-bound { color: #ed8936; }
.status-rest { color: #38b2ac; }
.status-normal { color: #68d391; }

.health-detail {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.health-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.value-with-bar {
  display: flex;
  align-items: center;
  gap: 12px;
}

.progress-bar {
  flex: 1;
  height: 20px;
  background: #f7fafc;
  border-radius: 10px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  transition: width 0.3s ease;
}

.progress-fill.life {
  background: linear-gradient(90deg, #f56565, #68d391);
}

.progress-fill.strength {
  background: linear-gradient(90deg, #4299e1, #9f7aea);
}

.effect-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px;
  background: #fed7d7;
  border-radius: 6px;
}

.effect-icon {
  font-size: 20px;
}

.equipment-detail {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.equipment-category {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.equipment-list {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.equipment-item {
  padding: 8px;
  border: 1px solid #e2e8f0;
  border-radius: 4px;
  background: #f7fafc;
}

.inventory-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(120px, 1fr));
  gap: 8px;
}

.inventory-item {
  padding: 8px;
  border: 1px solid #e2e8f0;
  border-radius: 4px;
  background: #f7fafc;
}

.activity-info {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.activity-item {
  display: flex;
  align-items: center;
  gap: 8px;
}

.activity-item label {
  font-weight: 500;
  min-width: 80px;
}

.no-item {
  color: #a0aec0;
  font-style: italic;
}
</style>