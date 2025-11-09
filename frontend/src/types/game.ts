// 游戏状态
export const GameStatus = {
  WAITING: 'waiting',
  RUNNING: 'running',
  PAUSED: 'paused',
  ENDED: 'ended',
  HIDDEN: 'hidden',
  DELETED: 'deleted'
} as const

export type GameStatus = typeof GameStatus[keyof typeof GameStatus]

// 游戏筛选类型
export const GameFilterType = {
  ALL: 'all',           // 全部（不包括已隐藏）
  ACTIVE: 'active',     // 活动中（等待中、进行中、已暂停）
  WAITING: 'waiting',   // 等待中
  RUNNING: 'running',   // 进行中
  ENDED: 'ended',       // 已结束
  HIDDEN: 'hidden'      // 已隐藏
} as const

export type GameFilterType = typeof GameFilterType[keyof typeof GameFilterType]

// 游戏基础信息
export interface Game {
  id: string
  name: string
  description?: string
  status: GameStatus
  player_count: number
  max_players: number
  created_at: string
  updated_at: string
  // 修改：直接包含规则配置而非模板信息
  rules_config?: Record<string, any>
}

// 游戏列表项
export interface GameListItem {
  id: string
  name: string
  description?: string
  status: GameStatus
  player_count: number
  max_players: number
  created_at: string
  director_password?: string
  // 修改：移除 rule_template 字段
}

// 游戏规则配置视图（公开接口返回）
export interface GameRulesConfigView {
  id: string
  name: string
  description?: string | null
  status: GameStatus
  rules_config: unknown
}

// 游戏详情（包含规则信息）
export interface GameWithRules extends GameListItem {
  // 修改：直接包含规则配置而非模板信息
  rules_config?: Record<string, any>
}

// 创建游戏请求
export interface CreateGameRequest {
  id: string
  name: string
  description?: string
  director_password: string
  max_players: number
  rule_template_id: string
}

// 更新游戏请求
export interface UpdateGameRequest {
  name?: string
  description?: string
  director_password?: string
  max_players?: number
  // 修改：移除 rule_template_id，添加 rules_config（内部使用）
  rules_config?: Record<string, any>
}

// 游戏列表查询参数
export interface GameListQuery {
  filter?: GameFilterType
}

// 游戏登录请求
export interface GameLoginRequest {
  password: string
}

// 游戏登录响应
export interface GameLoginResponse {
  success: boolean
  role: 'player' | 'director'
  message?: string
}

// 游戏身份验证响应
export interface GameAuthenticationResponse {
  role: 'actor' | 'director' | 'invalid'
  actor_id?: string | null
  actor_name?: string | null
}

// API响应基础类型
export interface ApiResponse<T> {
  success: boolean
  data?: T
  message?: string
  error?: string
}

// 击杀记录
export interface KillRecord {
  id: string
  game_id: string
  killer_id: string | null
  victim_id: string
  kill_time: string
  cause: string
  weapon: string | null
  location: string | null
}

// 消息记录
export interface MessageRecord {
  id: string
  game_id: string
  type: 'SystemNotice' | 'UserDirected' | 'Info'
  message: string
  player_id: string
  timestamp: string
  visible_to_all_players: boolean
  visible_to_director: boolean
}
