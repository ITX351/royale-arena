// API 基础配置
export const API_CONFIG = {
  BASE_URL: import.meta.env.PROD 
    ? '/royale-arena/api'   // 生产环境API路径
    : '/royale-arena/api',   // 开发环境API路径
  TIMEOUT: 30000
}

// API endpoints
export const API_ENDPOINTS = {
  // 游戏相关
  GAMES: '/games',
  GAME_DETAIL: (id: string) => `/games/${id}`,
  
  // 管理员相关
  ADMIN_LOGIN: '/admin/login',
  ADMIN_GAMES: '/admin/games',
  ADMIN_RULES: '/admin/rule-templates',
  ADMIN_USERS: '/admin/users',
  
  // 游戏参与
  JOIN_AS_PLAYER: (id: string) => `/games/${id}/join/player`,
  JOIN_AS_DIRECTOR: (id: string) => `/games/${id}/join/director`
}