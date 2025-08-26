import type { GameListItem, GameStatus, GameFilterType } from '@/types/game'

// 获取游戏状态的显示配置
export function getGameStatusConfig(status: GameStatus) {
  switch (status) {
    case 'waiting':
      return {
        text: '等待中',
        type: 'info' as const,
        color: '#909399'
      }
    case 'running':
      return {
        text: '进行中',
        type: 'success' as const,
        color: '#67C23A'
      }
    case 'paused':
      return {
        text: '已暂停',
        type: 'warning' as const,
        color: '#E6A23C'
      }
    case 'ended':
      return {
        text: '已结束',
        type: 'info' as const,
        color: '#606266'
      }
    case 'hidden':
      return {
        text: '已隐藏',
        type: 'info' as const,
        color: '#C0C4CC'
      }
    case 'deleted':
      return {
        text: '已删除',
        type: 'danger' as const,
        color: '#F56C6C'
      }
    default:
      return {
        text: '未知',
        type: 'info' as const,
        color: '#909399'
      }
  }
}

// 根据筛选类型过滤游戏列表
export function filterGamesByStatus(
  games: GameListItem[], 
  filter: GameFilterType, 
  searchQuery: string = ''
): GameListItem[] {
  let filtered = games

  // 按状态筛选
  switch (filter) {
    case 'all':
      // 全部（不包括已隐藏和已删除）
      filtered = games.filter(game => 
        !['hidden', 'deleted'].includes(game.status)
      )
      break
    case 'active':
      // 活动中（等待中、进行中、已暂停）
      filtered = games.filter(game => 
        ['waiting', 'running', 'paused'].includes(game.status)
      )
      break
    case 'waiting':
      filtered = games.filter(game => game.status === 'waiting')
      break
    case 'running':
      filtered = games.filter(game => game.status === 'running')
      break
    case 'ended':
      filtered = games.filter(game => game.status === 'ended')
      break
    case 'hidden':
      filtered = games.filter(game => game.status === 'hidden')
      break
    default:
      break
  }

  // 按搜索关键字筛选
  if (searchQuery.trim()) {
    const query = searchQuery.toLowerCase().trim()
    filtered = filtered.filter(game => 
      game.name.toLowerCase().includes(query) ||
      (game.description && game.description.toLowerCase().includes(query))
    )
  }

  return filtered
}

// 格式化日期时间
export function formatDateTime(dateString: string): string {
  const date = new Date(dateString)
  return date.toLocaleString('zh-CN', {
    year: 'numeric',
    month: '2-digit', 
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit'
  })
}

// 获取游戏状态样式类名
export function getGameStatusClass(status: GameStatus): string {
  return `game-card--${status}`
}