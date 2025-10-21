import apiClient from './client'
import { API_ENDPOINTS } from './config'
import type { 
  PlayerInfo,
  BatchAddPlayersRequest,
  BatchDeletePlayersRequest,
  BatchOperationResponse,
  DeleteSuccessInfo,
  DirectorAuthResponse,
  BatchPasteData,
  AddPlayerItem,
  ApiResponse
} from '@/types/director'

export class DirectorService {
  /**
   * 获取演员列表
   */
  async getPlayers(
    gameId: string, 
    password: string
  ): Promise<DirectorAuthResponse> {
    const response = await apiClient.get(
      API_ENDPOINTS.DIRECTOR_PLAYERS(gameId),
      { params: { password } }
    )
    return response.data
  }

  /**
   * 批量添加演员
   */
  async batchAddPlayers(
    gameId: string,
    password: string,
    players: AddPlayerItem[]
  ): Promise<ApiResponse<BatchOperationResponse<PlayerInfo>>> {
    const request: BatchAddPlayersRequest = {
      players: players.map(p => ({
        player_name: p.player_name,
        password: p.password,
        team_id: p.team_id
      }))
    }
    
    const response = await apiClient.post(
      API_ENDPOINTS.DIRECTOR_BATCH_ADD(gameId),
      request,
      { params: { password } }
    )
    return response.data
  }

  /**
   * 批量删除演员
   */
  async batchDeletePlayers(
    gameId: string,
    password: string,
    playerIds: string[]
  ): Promise<ApiResponse<BatchOperationResponse<DeleteSuccessInfo>>> {
    const request: BatchDeletePlayersRequest = {
      player_ids: playerIds
    }
    
    const response = await apiClient.delete(
      API_ENDPOINTS.DIRECTOR_BATCH_DELETE(gameId),
      { 
        data: request,
        params: { password }
      }
    )
    return response.data
  }

  /**
   * URI参数解析 - 从路径中提取密码
   */
  parsePasswordFromURI(path: string): string | null {
    // 匹配 /game/{gameId}/{password}
    const match = path.match(/\/game\/([^/]+)\/([^/]+)$/)
    return match ? decodeURIComponent(match[2]) : null
  }

  /**
   * 手动存盘游戏
   */
  async manualSaveGame(
    gameId: string,
    password: string
  ): Promise<any> {
    const response = await apiClient.post(
      API_ENDPOINTS.DIRECTOR_SAVE_GAME(gameId),
      { password }
    )
    return response.data
  }

  /**
   * 获取存档文件列表
   */
  async listSaveFiles(
    gameId: string,
    password: string
  ): Promise<any> {
    const response = await apiClient.get(
      API_ENDPOINTS.DIRECTOR_LIST_SAVES(gameId),
      { params: { password } }
    )
    return response.data
  }

  /**
   * 更新游戏状态
   */
  async updateGameStatus(
    gameId: string,
    password: string,
    status: string,
    saveFileName?: string
  ): Promise<any> {
    const requestData: any = {
      password,
      status
    }
    
    if (saveFileName) {
      requestData.save_file_name = saveFileName
    }
    
    const response = await apiClient.put(
      API_ENDPOINTS.DIRECTOR_UPDATE_STATUS(gameId),
      requestData
    )
    return response.data
  }

  /**
   * 导演编辑游戏属性
   */
  async editGame(
    gameId: string,
    password: string,
    data: {
      name?: string
      description?: string
      max_players?: number
      rules_config?: any
    }
  ): Promise<any> {
    const response = await apiClient.put(
      API_ENDPOINTS.DIRECTOR_EDIT_GAME(gameId),
      data,
      { params: { password } }
    )
    return response.data
  }

  /**
   * 解析批量粘贴数据
   */
  parseBatchPasteData(
    usernames: string,
    passwords: string,
    teamIds: string
  ): BatchPasteData {
    const usernameList = usernames.trim()
      .split('\n')
      .map(s => s.trim())
      .filter(s => s.length > 0)
    
    const passwordList = passwords.trim()
      .split('\n')
      .map(s => s.trim())
      .filter(s => s.length > 0)
    
    const teamIdList = teamIds.trim()
      .split('\n')
      .map(s => s.trim())
    
    // 数据一致性检查
    if (usernameList.length === 0) {
      return {
        usernames: [],
        passwords: [],
        teamIds: [],
        isValid: false,
        errorMessage: '用户名列表不能为空'
      }
    }
    
    if (passwordList.length === 0) {
      return {
        usernames: [],
        passwords: [],
        teamIds: [],
        isValid: false,
        errorMessage: '密码列表不能为空'
      }
    }
    
    if (usernameList.length !== passwordList.length) {
      return {
        usernames: [],
        passwords: [],
        teamIds: [],
        isValid: false,
        errorMessage: `用户名和密码数量不一致：用户名${usernameList.length}个，密码${passwordList.length}个`
      }
    }
    
    // 检查用户名重复
    const duplicateUsernames = this.findDuplicates(usernameList)
    if (duplicateUsernames.length > 0) {
      return {
        usernames: [],
        passwords: [],
        teamIds: [],
        isValid: false,
        errorMessage: `用户名重复：${duplicateUsernames.join(', ')}`
      }
    }
    
    // 检查密码重复
    const duplicatePasswords = this.findDuplicates(passwordList)
    if (duplicatePasswords.length > 0) {
      return {
        usernames: [],
        passwords: [],
        teamIds: [],
        isValid: false,
        errorMessage: `密码重复：${duplicatePasswords.join(', ')}`
      }
    }
    
    // 检查用户名和密码是否有交叉重复
    const crossDuplicates = usernameList.filter(username => passwordList.includes(username))
    if (crossDuplicates.length > 0) {
      return {
        usernames: [],
        passwords: [],
        teamIds: [],
        isValid: false,
        errorMessage: `用户名与密码不能相同：${crossDuplicates.join(', ')}`
      }
    }
    
    // 组队编号可以少于用户名数量，缺失的补空字符串
    const normalizedTeamIds = [...teamIdList]
    while (normalizedTeamIds.length < usernameList.length) {
      normalizedTeamIds.push('')
    }
    
    // 只取前N个组队编号（N为用户名数量）
    const finalTeamIds = normalizedTeamIds.slice(0, usernameList.length)
    
    return {
      usernames: usernameList,
      passwords: passwordList,
      teamIds: finalTeamIds,
      isValid: true
    }
  }

  /**
   * 查找数组中的重复项
   */
  private findDuplicates(array: string[]): string[] {
    const seen = new Set<string>()
    const duplicates = new Set<string>()
    
    for (const item of array) {
      if (seen.has(item)) {
        duplicates.add(item)
      } else {
        seen.add(item)
      }
    }
    
    return Array.from(duplicates)
  }

  /**
   * 检查新用户名和密码与现有用户是否重复
   */
  checkDuplicatesWithExistingUsers(
    newUsernames: string[],
    newPasswords: string[],
    existingPlayers: Array<{ name: string; password: string }>
  ): { isValid: boolean; errorMessage?: string } {
    const existingUsernames = existingPlayers.map(p => p.name)
    const existingPasswords = existingPlayers.map(p => p.password)
    
    // 检查新用户名与现有用户名是否重复
    const duplicateUsernames = newUsernames.filter(username => existingUsernames.includes(username))
    if (duplicateUsernames.length > 0) {
      return {
        isValid: false,
        errorMessage: `用户名与现有用户重复：${duplicateUsernames.join(', ')}`
      }
    }
    
    // 检查新密码与现有密码是否重复
    const duplicatePasswords = newPasswords.filter(password => existingPasswords.includes(password))
    if (duplicatePasswords.length > 0) {
      return {
        isValid: false,
        errorMessage: `密码与现有用户重复：${duplicatePasswords.join(', ')}`
      }
    }
    
    // 检查新用户名与现有密码是否重复
    const usernamePasswordConflicts = newUsernames.filter(username => existingPasswords.includes(username))
    if (usernamePasswordConflicts.length > 0) {
      return {
        isValid: false,
        errorMessage: `用户名与现有密码重复：${usernamePasswordConflicts.join(', ')}`
      }
    }
    
    // 检查新密码与现有用户名是否重复
    const passwordUsernameConflicts = newPasswords.filter(password => existingUsernames.includes(password))
    if (passwordUsernameConflicts.length > 0) {
      return {
        isValid: false,
        errorMessage: `密码与现有用户名重复：${passwordUsernameConflicts.join(', ')}`
      }
    }
    
    return { isValid: true }
  }
  validatePlayerPassword(password: string): { isValid: boolean; message?: string } {
    if (!password) {
      return { isValid: false, message: '密码不能为空' }
    }
    
    if (password.length === 0 || password.length > 40) {
      return { isValid: false, message: '密码长度必须在1-40位之间' }
    }
    
    if (!/^[a-zA-Z0-9]+$/.test(password)) {
      return { isValid: false, message: '密码只能包含字母和数字' }
    }
    
    return { isValid: true }
  }

  /**
   * 验证演员姓名格式
   */
  validatePlayerName(name: string): { isValid: boolean; message?: string } {
    if (!name.trim()) {
      return { isValid: false, message: '姓名不能为空' }
    }
    
    if (name.trim().length > 50) {
      return { isValid: false, message: '姓名长度不能超过50个字符' }
    }
    
    return { isValid: true }
  }

  /**
   * 生成临时ID
   */
  generateTempId(): string {
    return 'temp_' + Math.random().toString(36).substr(2, 9)
  }

  /**
   * 生成随机密码
   */
  generateRandomPassword(): string {
    const chars = 'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789'
    let result = ''
    for (let i = 0; i < 6; i++) {
      result += chars.charAt(Math.floor(Math.random() * chars.length))
    }
    return result
  }
}

// 创建单例实例
export const directorService = new DirectorService()