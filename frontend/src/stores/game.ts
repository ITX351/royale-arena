import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { gameService } from '@/services/gameService'
import { filterGamesByStatus } from '@/utils/gameFilter'
import type { GameListItem, GameFilterType } from '@/types/game'

export const useGameStore = defineStore('game', () => {
  // 状态
  const games = ref<GameListItem[]>([])
  const loading = ref(false)
  const searchQuery = ref('')
  const statusFilter = ref<GameFilterType>('all' as GameFilterType)
  const error = ref<string | null>(null)

  // 计算属性
  const filteredGames = computed(() => {
    return filterGamesByStatus(games.value, statusFilter.value, searchQuery.value)
  })

  // 操作
  const loadGames = async () => {
    loading.value = true
    error.value = null
    
    try {
      const response = await gameService.getGames({ filter: statusFilter.value })
      if (response.success && response.data) {
        games.value = response.data
      } else {
        throw new Error(response.error?.message || '获取游戏列表失败')
      }
    } catch (err) {
      console.error('加载游戏列表失败:', err)
      error.value = err instanceof Error ? err.message : '加载游戏列表失败'
    } finally {
      loading.value = false
    }
  }

  const refreshGames = async () => {
    await loadGames()
  }

  const setSearchQuery = (query: string) => {
    searchQuery.value = query
  }

  const setStatusFilter = (filter: GameFilterType) => {
    statusFilter.value = filter
  }

  const clearError = () => {
    error.value = null
  }

  return {
    // 状态
    games,
    loading,
    searchQuery,
    statusFilter,
    error,
    // 计算属性
    filteredGames,
    // 操作
    loadGames,
    refreshGames,
    setSearchQuery,
    setStatusFilter,
    clearError
  }
})