import axios from 'axios'
import { API_CONFIG } from './config'

const adminClient = axios.create({
  baseURL: API_CONFIG.BASE_URL,
  timeout: API_CONFIG.TIMEOUT,
  headers: {
    'Content-Type': 'application/json'
  }
})

// 请求拦截器 - 始终添加认证token
adminClient.interceptors.request.use(config => {
  const token = localStorage.getItem('admin_token')
  if (token) {
    config.headers.Authorization = `Bearer ${token}`
  }
  return config
})

// 响应拦截器 - 处理认证错误
adminClient.interceptors.response.use(
  response => response,
  error => {
    if (error.response?.status === 401) {
      // 只清除token，不进行自动重定向
      // 重定向由路由守卫处理
      localStorage.removeItem('admin_token')
      localStorage.removeItem('admin_user')
      
      console.warn('管理员token已过期，已清除本地存储')
    }
    return Promise.reject(error)
  }
)

export default adminClient