import { ElMessage } from 'element-plus'
import { directorService } from './directorService'
import type { GameListItem } from '@/types/game'

export interface AuthResult {
  success: boolean
  role?: 'director' | 'actor' | 'invalid'
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
    // 使用新的认证接口进行权限验证
    const authResult = await directorService.authenticateGame(gameId, password)
    
    if (authResult === 'director') {
      return {
        success: true,
        role: 'director'
      }
    } else if (authResult === 'actor') {
      return {
        success: true,
        role: 'actor'
      }
    } else {
      return {
        success: false,
        role: 'invalid',
        errorMessage: '密码错误'
      }
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
    router.push(`/game/${gameId}/${encodeURIComponent(password)}`)
    return true
  } else if (authResult.role === 'actor') {
    // 如果是演员身份，显示成功消息
    ElMessage.success('成功以演员身份登录')
    // 这里可以添加演员页面的跳转逻辑
    return true
  }
  
  return false
}