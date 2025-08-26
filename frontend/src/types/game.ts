// 游戏状态枚举
export enum GameStatus {
  WAITING = 'waiting',
  RUNNING = 'running',
  PAUSED = 'paused',
  ENDED = 'ended',
  HIDDEN = 'hidden',
  DELETED = 'deleted'
}

// 游戏筛选类型
export enum GameFilterType {
  ALL = 'all',           // 全部（不包括已隐藏）
  ACTIVE = 'active',     // 活动中（等待中、进行中、已暂停）
  WAITING = 'waiting',   // 等待中
  RUNNING = 'running',   // 进行中
  ENDED = 'ended',       // 已结束
  HIDDEN = 'hidden'      // 已隐藏
}

// 规则模版信息
export interface RuleTemplateInfo {
  id: string
  template_name: string
  description?: string
  rules_config: Record<string, any>
}

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
  rule_template?: RuleTemplateInfo
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
}

// 游戏详情（包含规则信息）
export interface GameWithRules extends GameListItem {
  rule_template?: RuleTemplateInfo
}

// 创建游戏请求
export interface CreateGameRequest {
  name: string
  description?: string
  director_password: string
  max_players: number
  rule_template_id?: string
}

// 更新游戏请求
export interface UpdateGameRequest {
  name?: string
  description?: string
  director_password?: string
  max_players?: number
  rule_template_id?: string
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