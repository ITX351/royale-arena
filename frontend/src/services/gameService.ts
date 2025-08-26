import apiClient from './client'
import adminClient from './adminClient'
import { API_ENDPOINTS } from './config'
import type { 
  GameListItem, 
  GameWithRules, 
  GameListQuery, 
  CreateGameRequest, 
  UpdateGameRequest,
  GameLoginRequest,
  GameLoginResponse,
  ApiResponse 
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

  // 以玩家身份加入游戏
  async joinAsPlayer(id: string, password: string): Promise<GameLoginResponse> {
    const response = await apiClient.post(API_ENDPOINTS.JOIN_AS_PLAYER(id), { password })
    return response.data
  },

  // 以导演身份加入游戏
  async joinAsDirector(id: string, password: string): Promise<GameLoginResponse> {
    const response = await apiClient.post(API_ENDPOINTS.JOIN_AS_DIRECTOR(id), { password })
    return response.data
  }
}