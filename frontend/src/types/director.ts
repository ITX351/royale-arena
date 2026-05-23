// 导演控制台相关类型定义

// 演员信息
export interface PlayerInfo {
  id: string
  name: string
  password: string
  game_id: string
  team_id: number
}

// 创建演员请求
export interface CreatePlayerRequest {
  player_name: string
  password: string
  team_id?: number
}

// 批量添加演员请求
export interface BatchAddPlayersRequest {
  players: CreatePlayerRequest[]
}

// 批量删除演员请求
export interface BatchDeletePlayersRequest {
  player_ids: string[]
}

// 操作失败信息
export interface OperationFailure {
  player_name?: string
  id?: string
  reason: string
}

// 批量操作响应
export interface BatchOperationResponse<T> {
  success: T[]
  failed: OperationFailure[]
}

// 演员列表响应
export interface PlayersListResponse {
  players: PlayerInfo[]
}

// 删除操作成功信息
export interface DeleteSuccessInfo {
  id: string
  name: string
  message: string
}

// 导演认证响应
export interface DirectorAuthResponse {
  success: boolean
  data: PlayersListResponse
}

// 批量粘贴数据处理
export interface BatchPasteData {
  usernames: string[]
  passwords: string[]
  teamIds: string[]
  isValid: boolean
  errorMessage?: string
}

// 导演控制台状态
export interface DirectorConsoleState {
  // URI参数相关
  passwordFromURI: string | null
  autoAuthenticated: boolean
  
  // 认证相关
  isAuthenticated: boolean
  directorPassword: string
  authLoading: boolean
  authError: string | null
  
  // 演员数据
  players: PlayerInfo[]
  playersLoading: boolean
  
  // UI状态
  playersTableCollapsed: boolean
  
  // 批量操作状态
  selectedPlayers: string[]
  batchAddDialogVisible: boolean
  batchDeleteLoading: boolean
  batchAddLoading: boolean
  
  // 批量添加表单
  addPlayersForm: AddPlayerItem[]
  
  // 批量粘贴功能
  batchPasteDialogVisible: boolean
  pasteUsernames: string
  pastePasswords: string
  pasteTeamIds: string
}

// 添加演员表单项
export interface AddPlayerItem {
  player_name: string
  password: string
  team_id?: number
  tempId: string // 临时ID，用于表单管理
}

// API响应基础类型
export interface ApiResponse<T> {
  success: boolean
  data: T
  message?: string
}