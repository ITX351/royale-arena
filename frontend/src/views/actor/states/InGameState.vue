<template>
  <el-card class="in-game-state">
    <template #header>
      <div class="card-header">
        <h3>游戏进行中</h3>
      </div>
    </template>
    
    <div class="state-content">
      <el-alert
        title="游戏正在进行中"
        type="success"
        description="请根据游戏规则进行操作"
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
          <el-descriptions-item label="当前状态">
            <el-tag type="success">{{ formatGameStatus(game.status) }}</el-tag>
          </el-descriptions-item>
          <el-descriptions-item label="玩家数量">
            {{ game.player_count }} / {{ game.max_players }}
          </el-descriptions-item>
          <el-descriptions-item label="创建时间">
            {{ formatTimestamp(game.created_at) }}
          </el-descriptions-item>
        </el-descriptions>
      </div>
      
      <div class="player-actions" v-if="game">
        <h4>玩家操作</h4>
        <el-row :gutter="20">
          <el-col :span="12">
            <el-button type="primary" plain disabled>移动</el-button>
          </el-col>
          <el-col :span="12">
            <el-button type="primary" plain disabled>搜索</el-button>
          </el-col>
          <el-col :span="12">
            <el-button type="primary" plain disabled>攻击</el-button>
          </el-col>
          <el-col :span="12">
            <el-button type="primary" plain disabled>使用道具</el-button>
          </el-col>
        </el-row>
      </div>
    </div>
  </el-card>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { GameWithRules } from '@/types/game'
import { GameStatus } from '@/types/game'
import { formatTimestamp, formatGameStatus } from '@/utils/gameUtils'

const props = defineProps<{
  game: GameWithRules | null
}>()
</script>

<style scoped>
.in-game-state {
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

.player-actions h4 {
  margin-top: 0;
  margin-bottom: 15px;
}

.player-actions :deep(.el-button) {
  width: 100%;
  margin-bottom: 10px;
}
</style>