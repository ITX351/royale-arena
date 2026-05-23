<template>
  <el-card class="pre-game-state">
    <template #header>
      <div class="card-header">
        <h3>游戏准备状态</h3>
      </div>
    </template>
    
    <div class="state-content">
      <el-alert
        title="游戏尚未开始"
        type="info"
        description="请等待导演开始游戏"
        show-icon
        :closable="false"
      />
      
      <div class="game-info" v-if="game">
        <el-descriptions :column="1" border>
          <el-descriptions-item label="游戏ID">
            {{ game.id }}
          </el-descriptions-item>
          <el-descriptions-item label="游戏名称">
            {{ game.name }}
          </el-descriptions-item>
          <el-descriptions-item label="玩家名称" v-if="playerName">
            {{ playerName }}
          </el-descriptions-item>
          <el-descriptions-item label="玩家数量">
            {{ game.player_count }} / {{ game.max_players }}
          </el-descriptions-item>
          <el-descriptions-item label="创建时间">
            {{ formatTimestamp(game.created_at) }}
          </el-descriptions-item>
          <el-descriptions-item label="游戏状态">
            <el-tag type="info">等待开始</el-tag>
          </el-descriptions-item>
        </el-descriptions>
      </div>
    </div>
  </el-card>
</template>

<script setup lang="ts">
import { formatTimestamp } from '@/utils/gameUtils'
import type { GameWithRules } from '@/types/game'

defineProps<{
  game: GameWithRules | null
  playerName?: string | null
}>()
</script>

<style scoped>
.pre-game-state {
  height: 100%;
}

.card-header h3 {
  margin: 0;
  color: #303133;
}

.state-content {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.game-info {
  max-width: 600px;
}
</style>