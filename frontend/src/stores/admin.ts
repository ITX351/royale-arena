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
    
    if (savedToken && savedUser) {
      try {
        token.value = savedToken
        userInfo.value = JSON.parse(savedUser)
        
        // 验证token是否有效（可选）
        // 这里可以添加一个验证接口调用
        
        isLoggedIn.value = true
      } catch (err) {
        console.warn('清除无效的管理员登录信息:', err)
        // 清除无效数据
        logout()
      }
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
        throw new Error(response.message || '登录失败')
      }
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : '登录失败'
      error.value = errorMessage
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
    clearError
  }
})