// 管理员用户信息
export interface AdminUser {
  id: string
  username: string
  is_super_admin: boolean
  created_at: string
  updated_at: string
}

// 登录凭据
export interface LoginCredentials {
  username: string
  password: string
}

// 登录响应
export interface LoginResponse {
  success: boolean
  token: string
  user: AdminUser
  message?: string
}

// 创建管理员请求
export interface CreateAdminRequest {
  username: string
  password: string
  is_super_admin: boolean
}

// 更新管理员请求
export interface UpdateAdminRequest {
  username?: string
  password?: string
  is_super_admin?: boolean
}

// 重置密码请求
export interface ResetPasswordRequest {
  new_password: string
}

// 重置密码响应
export interface ResetPasswordResponse {
  success: boolean
  message: string
  user: AdminUser
}

// 规则模版
export interface RuleTemplate {
  id: string
  template_name: string
  description?: string
  is_active: boolean
  rules_config: Record<string, any>
  created_at: string
  updated_at: string
}

// 创建规则模版请求
export interface CreateRuleTemplateRequest {
  template_name: string
  description?: string
  rules_config: Record<string, any>
}

// 更新规则模版请求
export interface UpdateRuleTemplateRequest {
  template_name?: string
  description?: string
  is_active?: boolean
  rules_config?: Record<string, any>
}

// API响应通用格式
export interface ApiResponse<T = any> {
  success: boolean
  data?: T
  error?: {
    message: string
    details?: string
  }
}