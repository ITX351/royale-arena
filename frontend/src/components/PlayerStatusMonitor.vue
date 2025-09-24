<template>
  <div class="player-status-monitor">
    <!-- 玩家列表概览 -->
    <div class="players-overview">
      <div 
        v-for="player in players" 
        :key="player.id"
        class="player-card"
        :class="{
          'player-dead': !player.is_alive,
          'player-bound': player.is_bound,
          'player-bleeding': player.bleed_rounds_remaining > 0
        }"
        @click="selectPlayer(player.id)"
      >
        <!-- 基础信息 -->
        <div class="player-basic">
          <h4>{{ player.name }}</h4>
          <span class="location">{{ player.location || '未出生' }}</span>
          <span v-if="player.team_id" class="team">队伍{{ player.team_id }}</span>
        </div>
        
        <!-- 生命体力 -->
        <div class="player-health">
          <div class="health-bar">
            <span>生命: {{ player.life }}/{{ player.max_life }}</span>
            <div class="bar">
              <div 
                class="fill life-fill" 
                :style="{ width: (player.life / player.max_life * 100) + '%' }"
              ></div>
            </div>
          </div>
          <div class="strength-bar">
            <span>体力: {{ player.strength }}/{{ player.max_strength }}</span>
            <div class="bar">
              <div 
                class="fill strength-fill" 
                :style="{ width: (player.strength / player.max_strength * 100) + '%' }"
              ></div>
            </div>
          </div>
        </div>
        
        <!-- 状态效果 -->
        <div class="player-effects">
          <span v-if="!player.is_alive" class="status-dead">已死亡</span>
          <span v-if="player.is_bound" class="status-bound">被捆绑</span>
          <span v-if="player.rest_mode" class="status-rest">静养模式</span>
          <span v-if="player.bleed_rounds_remaining > 0" class="status-bleed">
            流血 {{ player.bleed_damage }}×{{ player.bleed_rounds_remaining }}
          </span>
        </div>
        
        <!-- 装备概览 -->
        <div class="equipment-summary">
          <span class="equipment-count">
            武器: {{ player.equipped_weapons.length }}/{{ maxWeapons }}
          </span>
          <span class="equipment-count">
            防具: {{ player.equipped_armors.length }}/{{ maxArmors }}
          </span>
          <span class="inventory-count">
            背包: {{ player.inventory.length }}/{{ maxBackpack }}
          </span>
        </div>
      </div>
    </div>
    
    <!-- 选中玩家详情 -->
    <div v-if="selectedPlayer" class="player-detail">
      <PlayerDetailPanel 
        :player="selectedPlayer" 
        :max-weapons="maxWeapons"
        :max-armors="maxArmors" 
        :max-backpack="maxBackpack"
      />
    </div>
    
    <!-- 玩家统计信息 -->
    <div class="player-stats">
      <h4>玩家统计</h4>
      <div class="stats-grid">
        <div class="stat-item">
          <span class="stat-label">总数:</span>
          <span class="stat-value">{{ playerStats.totalPlayers }}</span>
        </div>
        <div class="stat-item">
          <span class="stat-label">存活:</span>
          <span class="stat-value alive">{{ playerStats.alivePlayers }}</span>
        </div>
        <div class="stat-item">
          <span class="stat-label">死亡:</span>
          <span class="stat-value dead">{{ playerStats.deadPlayers }}</span>
        </div>
        <div class="stat-item">
          <span class="stat-label">捆绑:</span>
          <span class="stat-value bound">{{ playerStats.boundPlayers }}</span>
        </div>
        <div class="stat-item">
          <span class="stat-label">静养:</span>
          <span class="stat-value rest">{{ playerStats.restingPlayers }}</span>
        </div>
        <div class="stat-item">
          <span class="stat-label">流血:</span>
          <span class="stat-value bleed">{{ playerStats.bleedingPlayers }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import type { Player, PlayerStats } from '@/types/gameStateTypes'
import PlayerDetailPanel from './PlayerDetailPanel.vue'

interface Props {
  players: Player[]
  maxWeapons?: number
  maxArmors?: number
  maxBackpack?: number
}

const props = withDefaults(defineProps<Props>(), {
  maxWeapons: 1,
  maxArmors: 1,
  maxBackpack: 4
})

const selectedPlayerId = ref<string | null>(null)

const selectedPlayer = computed(() => {
  if (!selectedPlayerId.value) return null
  return props.players.find(p => p.id === selectedPlayerId.value) || null
})

const playerStats = computed((): PlayerStats => {
  const stats = {
    totalPlayers: props.players.length,
    alivePlayers: 0,
    deadPlayers: 0,
    boundPlayers: 0,
    restingPlayers: 0,
    bleedingPlayers: 0
  }
  
  props.players.forEach(player => {
    if (player.is_alive) {
      stats.alivePlayers++
    } else {
      stats.deadPlayers++
    }
    
    if (player.is_bound) {
      stats.boundPlayers++
    }
    
    if (player.rest_mode) {
      stats.restingPlayers++
    }
    
    if (player.bleed_rounds_remaining > 0) {
      stats.bleedingPlayers++
    }
  })
  
  return stats
})

const selectPlayer = (playerId: string) => {
  selectedPlayerId.value = selectedPlayerId.value === playerId ? null : playerId
}
</script>

<style scoped>
.player-status-monitor {
  display: flex;
  flex-direction: column;
  gap: 20px;
  padding: 16px;
}

.players-overview {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: 16px;
}

.player-card {
  background: white;
  border: 2px solid #e2e8f0;
  border-radius: 8px;
  padding: 16px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.player-card:hover {
  border-color: #4299e1;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}

.player-card.player-dead {
  border-color: #f56565;
  background-color: #fed7d7;
}

.player-card.player-bound {
  border-color: #ed8936;
  background-color: #feebc8;
}

.player-card.player-bleeding {
  border-left: 4px solid #e53e3e;
}

.player-basic {
  margin-bottom: 12px;
}

.player-basic h4 {
  margin: 0 0 4px 0;
  font-size: 16px;
  font-weight: 600;
  color: #2d3748;
}

.location {
  display: inline-block;
  background: #edf2f7;
  color: #4a5568;
  padding: 2px 8px;
  border-radius: 12px;
  font-size: 12px;
  margin-right: 8px;
}

.team {
  display: inline-block;
  background: #bee3f8;
  color: #2b6cb0;
  padding: 2px 8px;
  border-radius: 12px;
  font-size: 12px;
}

.player-health {
  margin-bottom: 12px;
}

.health-bar, .strength-bar {
  margin-bottom: 8px;
}

.health-bar span, .strength-bar span {
  display: block;
  font-size: 12px;
  margin-bottom: 4px;
  color: #4a5568;
}

.bar {
  height: 8px;
  background: #edf2f7;
  border-radius: 4px;
  overflow: hidden;
}

.fill {
  height: 100%;
  transition: width 0.3s ease;
}

.life-fill {
  background: linear-gradient(90deg, #f56565, #68d391);
}

.strength-fill {
  background: linear-gradient(90deg, #4299e1, #9f7aea);
}

.player-effects {
  margin-bottom: 12px;
  min-height: 20px;
}

.player-effects span {
  display: inline-block;
  padding: 2px 6px;
  border-radius: 12px;
  font-size: 10px;
  font-weight: 500;
  margin-right: 4px;
  margin-bottom: 4px;
}

.status-dead {
  background: #fed7d7;
  color: #c53030;
}

.status-bound {
  background: #feebc8;
  color: #c05621;
}

.status-rest {
  background: #c6f6d5;
  color: #25855a;
}

.status-bleed {
  background: #fed7d7;
  color: #c53030;
}

.equipment-summary {
  display: flex;
  justify-content: space-between;
  font-size: 12px;
  color: #4a5568;
}

.equipment-count, .inventory-count {
  font-weight: 500;
}

.player-detail {
  background: white;
  border-radius: 8px;
  border: 1px solid #e2e8f0;
}

.player-stats {
  background: white;
  border-radius: 8px;
  padding: 16px;
  border: 1px solid #e2e8f0;
}

.player-stats h4 {
  margin: 0 0 12px 0;
  color: #2d3748;
}

.stats-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(100px, 1fr));
  gap: 12px;
}

.stat-item {
  text-align: center;
}

.stat-label {
  display: block;
  font-size: 12px;
  color: #718096;
  margin-bottom: 4px;
}

.stat-value {
  display: block;
  font-size: 18px;
  font-weight: 600;
  color: #2d3748;
}

.stat-value.alive { color: #38a169; }
.stat-value.dead { color: #e53e3e; }
.stat-value.bound { color: #d69e2e; }
.stat-value.rest { color: #319795; }
.stat-value.bleed { color: #e53e3e; }
</style>