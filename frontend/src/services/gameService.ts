import apiClient from './client'
import adminClient from './adminClient'
import { API_ENDPOINTS } from './config'
import type {
  GameListItem,
  GameWithRules,
  GameListQuery,
  CreateGameRequest,
  UpdateGameRequest,
  ApiResponse,
  KillRecord,
  MessageRecord,
  GameRulesConfigView
} from '@/types/game'

export const gameService = {
  // 获取游戏列表
  async getGames(query?: GameListQuery, isAdmin: boolean = false): Promise<ApiResponse<GameListItem[]>> {
    const client = isAdmin ? adminClient : apiClient;
    const endpoint = isAdmin ? API_ENDPOINTS.ADMIN_GAMES : API_ENDPOINTS.GAMES;
    const response = await client.get(endpoint, { params: query })
    return response.data
  },

  // 获取游戏详情
  async getGameDetail(id: string, isAdmin: boolean = false): Promise<ApiResponse<GameWithRules>> {
    const client = isAdmin ? adminClient : apiClient;
    const endpoint = isAdmin ? `${API_ENDPOINTS.ADMIN_GAMES}/${id}` : API_ENDPOINTS.GAME_DETAIL(id);
    const response = await client.get(endpoint)
    return response.data
  },

  // 创建游戏（管理员）
  async createGame(data: CreateGameRequest): Promise<ApiResponse<GameWithRules>> {
    const response = await adminClient.post(API_ENDPOINTS.ADMIN_GAMES, data)
    return response.data
  },

  // 更新游戏（管理员）
  async updateGame(id: string, data: UpdateGameRequest): Promise<ApiResponse<GameWithRules>> {
    const response = await adminClient.put(`${API_ENDPOINTS.ADMIN_GAMES}/${id}`, data)
    return response.data
  },

  // 删除游戏（管理员）
  async deleteGame(id: string): Promise<ApiResponse<void>> {
    const response = await adminClient.delete(`${API_ENDPOINTS.ADMIN_GAMES}/${id}`)
    return response.data
  },

  // 获取导演消息记录
  async getDirectorMessages(gameId: string, password: string): Promise<ApiResponse<MessageRecord[]>> {
    const response = await apiClient.get(API_ENDPOINTS.DIRECTOR_MESSAGES(gameId), { 
      params: { password } 
    })
    return response.data
  },

  // 获取导演击杀记录
  async getDirectorKillRecords(gameId: string, password: string): Promise<ApiResponse<KillRecord[]>> {
    const response = await apiClient.get(API_ENDPOINTS.DIRECTOR_KILL_RECORDS(gameId), { 
      params: { password } 
    })
    return response.data
  },

  // 获取玩家消息记录
  async getPlayerMessages(gameId: string, playerId: string, password: string): Promise<ApiResponse<MessageRecord[]>> {
    const response = await apiClient.post(API_ENDPOINTS.PLAYER_MESSAGES(gameId, playerId), { password })
    return response.data
  },

  // 获取玩家击杀记录
  async getPlayerKillRecords(gameId: string, playerId: string, password: string): Promise<ApiResponse<KillRecord[]>> {
    const response = await apiClient.post(API_ENDPOINTS.PLAYER_KILL_RECORDS(gameId, playerId), { password })
    return response.data
  },

  // 获取带规则配置的游戏列表
  async getGamesRulesConfig(): Promise<ApiResponse<GameRulesConfigView[]>> {
    const response = await apiClient.get(API_ENDPOINTS.GAMES_RULES_CONFIG)
    return response.data
  }
}