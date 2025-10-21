import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { adminService } from '@/services/adminService'
import type { AdminUser, LoginCredentials } from '@/types/admin'

export const useAdminStore = defineStore('admin', () => {
  // 状态
  const isLoggedIn = ref(false)
  const userInfo = ref<AdminUser | null>(null)
  const token = ref<string | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)

  // 计算属性
  const isSuperAdmin = computed(() => {
    return userInfo.value?.is_super_admin || false
  })

  // 初始化检查登录状态
  const initAuth = async () => {
    const savedToken = localStorage.getItem('admin_token')
    const savedUser = localStorage.getItem('admin_user')
    
    // 严格检查数据有效性
    if (savedToken && savedUser && 
        savedToken !== 'null' && savedToken !== 'undefined' &&
        savedUser !== 'null' && savedUser !== 'undefined') {
      try {
        // 尝试解析用户信息
        const parsedUser = JSON.parse(savedUser)
        if (parsedUser && typeof parsedUser === 'object' && parsedUser.id) {
          token.value = savedToken
          userInfo.value = parsedUser
          isLoggedIn.value = true
          
          // 验证token是否有效（简单本地验证）
          await validateToken(savedToken)
        } else {
          throw new Error('用户信息格式无效')
        }
      } catch (err) {
        console.warn('清除无效的管理员登录信息:', err)
        // 清除无效数据
        logout()
      }
    }
  }
  
  // 验证token有效性（简化版本）
  const validateToken = async (token: string) => {
    try {
      // 基本格式检查
      if (!token || token.length < 10) {
        throw new Error('无效的token格式')
      }
      
      // JWT token基本格式检查（包含三个部分）
      const parts = token.split('.')
      if (parts.length !== 3) {
        throw new Error('无效的token格式')
      }
      
      // 暂时跳过复杂的payload解析，避免错误
      // 在后续版本中可以添加更详细的验证
      
    } catch (err) {
      console.warn('Token验证失败:', err)
      throw err
    }
  }

  // 登录操作
  const login = async (credentials: LoginCredentials) => {
    loading.value = true
    error.value = null
    
    try {
      const response = await adminService.login(credentials)
      
      if (response.success) {
        token.value = response.token
        userInfo.value = response.user
        isLoggedIn.value = true
        
        // 保存到本地存储
        localStorage.setItem('admin_token', response.token)
        localStorage.setItem('admin_user', JSON.stringify(response.user))
        
        return { success: true }
      } else {
        const friendlyMessage = response.message === 'Invalid credentials' || response.message === 'Unauthorized' 
          ? '用户名或密码错误，请检查后重试'
          : response.message || '登录失败，请稍后重试'
        throw new Error(friendlyMessage)
      }
    } catch (err: any) {
      let errorMessage = '登录失败，请稍后重试'
      
      // 根据HTTP状态码提供更友好的错误信息
      if (err.response?.status === 401) {
        errorMessage = '用户名或密码错误，请检查后重试'
      } else if (err.response?.status === 403) {
        errorMessage = '账户被禁用，请联系管理员'
      } else if (err.response?.status === 404) {
        errorMessage = '登录服务暂时不可用，请稍后重试'
      } else if (err.response?.status >= 500) {
        errorMessage = '服务器错误，请稍后重试'
      } else if (err.code === 'ECONNABORTED' || err.message?.includes('timeout')) {
        errorMessage = '请求超时，请检查网络连接后重试'
      } else if (err.code === 'NETWORK_ERROR' || err.message?.includes('Network Error')) {
        errorMessage = '网络连接失败，请检查网络设置后重试'
      } else if (err.message && !err.message.includes('Error') && !err.message.includes('Failed')) {
        // 如果有自定义的友好错误信息，使用它
        errorMessage = err.message
      }
      
      error.value = errorMessage
      
      // 确保错误信息在界面上保持显示，不会一闪而过
      console.error('管理员登录失败:', err)
      
      // 返回错误信息，让调用者可以处理
      return { success: false, error: errorMessage }
    } finally {
      loading.value = false
    }
  }

  // 登出操作
  const logout = () => {
    token.value = null
    userInfo.value = null
    isLoggedIn.value = false
    
    // 清除本地存储
    localStorage.removeItem('admin_token')
    localStorage.removeItem('admin_user')
  }

  // 清除错误
  const clearError = () => {
    error.value = null
  }

  // 重置当前管理员密码
  const resetPassword = async (newPassword: string) => {
    if (!userInfo.value) {
      return { success: false, message: '未找到管理员信息，请重新登录后再试' }
    }

    try {
      const response = await adminService.resetOwnPassword({ new_password: newPassword })

      if (response.success) {
        userInfo.value = { ...userInfo.value, ...response.user }
        localStorage.setItem('admin_user', JSON.stringify(response.user))
        return { success: true, message: response.message }
      }

      return {
        success: false,
        message: response.message || '密码重置失败，请稍后重试'
      }
    } catch (err: any) {
      const message = err.response?.data?.error?.message || err.message || '密码重置失败，请稍后重试'
      console.error('管理员密码重置失败:', err)
      return { success: false, message }
    }
  }

  return {
    // 状态
    isLoggedIn,
    userInfo,
    token,
    loading,
    error,
    // 计算属性
    isSuperAdmin,
    // 操作
    initAuth,
    login,
    logout,
    clearError,
    resetPassword
  }
})