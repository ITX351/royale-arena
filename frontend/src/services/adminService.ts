import adminClient from './adminClient'
import apiClient from './client'
import { API_ENDPOINTS } from './config'
import type {
  LoginCredentials,
  LoginResponse,
  AdminUser,
  CreateAdminRequest,
  UpdateAdminRequest,
  RuleTemplate,
  CreateRuleTemplateRequest,
  UpdateRuleTemplateRequest,
  ApiResponse,
  ResetPasswordRequest,
  ResetPasswordResponse
} from '@/types/admin'

export const adminService = {
  // 管理员登录
  async login(credentials: LoginCredentials): Promise<LoginResponse> {
    const response = await apiClient.post(API_ENDPOINTS.ADMIN_LOGIN, credentials)
    return response.data
  },

  // 获取管理员列表
  async getAdmins(): Promise<ApiResponse<AdminUser[]>> {
    const response = await adminClient.get(API_ENDPOINTS.ADMIN_USERS)
    return response.data
  },

  // 创建管理员
  async createAdmin(data: CreateAdminRequest): Promise<ApiResponse<AdminUser>> {
    const response = await adminClient.post(API_ENDPOINTS.ADMIN_USERS, data)
    return response.data
  },

  // 更新管理员
  async updateAdmin(id: string, data: UpdateAdminRequest): Promise<ApiResponse<AdminUser>> {
    const response = await adminClient.put(`${API_ENDPOINTS.ADMIN_USERS}/${id}`, data)
    return response.data
  },

  // 删除管理员
  async deleteAdmin(id: string): Promise<ApiResponse<void>> {
    const response = await adminClient.delete(`${API_ENDPOINTS.ADMIN_USERS}/${id}`)
    return response.data
  },

  // 自助重置密码
  async resetOwnPassword(data: ResetPasswordRequest): Promise<ResetPasswordResponse> {
    const response = await adminClient.put(API_ENDPOINTS.ADMIN_RESET_PASSWORD, data)
    return response.data
  },

  // 获取规则模版列表
  async getRuleTemplates(): Promise<ApiResponse<RuleTemplate[]>> {
    const response = await adminClient.get(API_ENDPOINTS.ADMIN_RULES)
    return response.data
  },

  // 创建规则模版
  async createRuleTemplate(data: CreateRuleTemplateRequest): Promise<ApiResponse<RuleTemplate>> {
    const response = await adminClient.post(API_ENDPOINTS.ADMIN_RULES, data)
    return response.data
  },

  // 更新规则模版
  async updateRuleTemplate(id: string, data: UpdateRuleTemplateRequest): Promise<ApiResponse<RuleTemplate>> {
    const response = await adminClient.put(`${API_ENDPOINTS.ADMIN_RULES}/${id}`, data)
    return response.data
  },

  // 删除规则模版
  async deleteRuleTemplate(id: string): Promise<ApiResponse<void>> {
    const response = await adminClient.delete(`${API_ENDPOINTS.ADMIN_RULES}/${id}`)
    return response.data
  }
}