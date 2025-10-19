<template>
  <el-card class="other-state">
    <template #header>
      <div class="card-header">
        <h3>游戏其他状态</h3>
      </div>
    </template>
    
    <div class="state-content">
      <el-alert
        v-if="game && (game.status === 'ended')"
        title="游戏已结束"
        type="warning"
        description="感谢参与游戏"
        show-icon
        :closable="false"
      />
      
      <el-alert
        v-else-if="game && (game.status === 'hidden')"
        title="游戏已隐藏"
        type="info"
        description="该游戏已被导演隐藏"
        show-icon
        :closable="false"
      />
      
      <el-alert
        v-else-if="game && (game.status === 'deleted')"
        title="游戏已删除"
        type="error"
        description="该游戏已被导演删除"
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
            <el-tag :type="getStatusTagType(game.status)">{{ formatGameStatus(game.status) }}</el-tag>
          </el-descriptions-item>
          <el-descriptions-item label="玩家数量">
            {{ game.player_count }} / {{ game.max_players }}
          </el-descriptions-item>
          <el-descriptions-item label="创建时间">
            {{ formatTimestamp(game.created_at) }}
          </el-descriptions-item>
        </el-descriptions>
      </div>
    </div>
  </el-card>
</template>

<script setup lang="ts">
import { formatTimestamp, formatGameStatus, getStatusTagType } from '@/utils/gameUtils'
import type { GameWithRules } from '@/types/game'

defineProps<{
  game: GameWithRules | null
}>()
</script>

<style scoped>
.other-state {
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