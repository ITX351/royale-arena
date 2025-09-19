<template>
  <div class="pre-game-management">
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
  </div>
</template>

<script setup lang="ts">
import { defineProps, defineEmits } from 'vue'
import type { GameWithRules } from '@/types/game'
import ActorManagement from './ActorManagement.vue'
import RuleManagement from './RuleManagement.vue'

// Props
const props = defineProps<{
  game: GameWithRules
  directorPassword: string
}>()

// Emits
defineEmits<{
  (e: 'refresh'): void
}>()
</script>

<style scoped>
.pre-game-management {
  margin-top: 24px;
}

@media (max-width: 768px) {
  .pre-game-management {
    margin-top: 16px;
  }
}
</style>