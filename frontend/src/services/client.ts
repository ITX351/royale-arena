import axios from 'axios'
import { API_CONFIG } from './config'

const apiClient = axios.create({
  baseURL: API_CONFIG.BASE_URL,
  timeout: API_CONFIG.TIMEOUT,
  headers: {
    'Content-Type': 'application/json'
  }
})

// 请求拦截器 - 添加认证token
apiClient.interceptors.request.use(config => {
  // 只在访问管理员API时才添加token
  const isAdminAPI = config.url?.includes('/admin/')
  if (isAdminAPI) {
    const token = localStorage.getItem('admin_token')
    if (token) {
      config.headers.Authorization = `Bearer ${token}`
    }
  }
  return config
})

// 响应拦截器 - 处理错误
apiClient.interceptors.response.use(
  response => response,
  error => {
    if (error.response?.status === 401) {
      // 清除过期token
      localStorage.removeItem('admin_token')
      localStorage.removeItem('admin_user')
      
      // 只在访问管理员相关API时才重定向到登录页
      // 但要排除登录页面本身，避免登录失败时页面刷新
      const isAdminAPI = error.config?.url?.includes('/admin/')
      const isLoginPage = window.location.pathname === '/admin/login'
      if (isAdminAPI && window.location.pathname.startsWith('/admin') && !isLoginPage) {
        window.location.href = '/admin/login'
      }
    }
    return Promise.reject(error)
  }
)

export default apiClient