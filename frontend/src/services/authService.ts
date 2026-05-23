import { ElMessage } from 'element-plus'
import apiClient from './client'
import { API_ENDPOINTS } from './config'
import type { GameAuthenticationResponse } from '@/types/game'

export interface AuthResult {
  success: boolean
  role?: 'director' | 'actor' | 'invalid'
  actorId?: string
  actorName?: string
  errorMessage?: string
}

/**
 * 统一的游戏认证函数
 * @param gameId 游戏ID
 * @param password 密码
 * @returns 认证结果
 */
export async function authenticateGame(gameId: string, password: string): Promise<AuthResult> {
  if (!password.trim()) {
    return {
      success: false,
      errorMessage: '请输入密码'
    }
  }

  try {
    // 使用认证接口进行权限验证
    const response = await apiClient.get<GameAuthenticationResponse>(
      API_ENDPOINTS.GAME_AUTH(gameId),
      { params: { password } }
    )

    const {
      role,
      actor_id: actorIdRaw,
      actor_name: actorNameRaw
    } = response.data

    if (role === 'director') {
      return {
        success: true,
        role: 'director'
      }
    } else if (role === 'actor') {
      return {
        success: true,
        role: 'actor',
        actorId: actorIdRaw ?? undefined,
        actorName: actorNameRaw ?? undefined
      }
    } else if (role === 'invalid') {
      return {
        success: false,
        role: 'invalid',
        errorMessage: '密码错误'
      }
    }
    
    return {
      success: false,
      errorMessage: '未知的身份验证结果'
    }
  } catch (error) {
    console.error('登录失败:', error)
    return {
      success: false,
      errorMessage: '登录失败，请稍后重试'
    }
  }
}

/**
 * 处理认证结果并执行相应的路由跳转
 * @param authResult 认证结果
 * @param gameId 游戏ID
 * @param password 密码
 * @param router Vue Router实例
 */
export function handleAuthResult(
  authResult: AuthResult, 
  gameId: string, 
  password: string, 
  router: any
): boolean {
  if (!authResult.success) {
    if (authResult.errorMessage) {
      ElMessage.error(authResult.errorMessage)
    }
    return false
  }

  if (authResult.role === 'director') {
    // 如果是导演身份，跳转到导演页面
    ElMessage.success('成功以导演身份进入控制台')
    router.push(`/game/${gameId}/director/${encodeURIComponent(password)}`)
    return true
  } else if (authResult.role === 'actor') {
    // 如果是演员身份，跳转到演员页面
    const actorName = authResult.actorName?.trim() || '演员'
    ElMessage.success(`成功以演员身份登录：${actorName}`)
    router.push(`/game/${gameId}/actor/${encodeURIComponent(password)}`)
    return true
  }
  
  return false
}