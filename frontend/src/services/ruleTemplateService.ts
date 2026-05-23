import apiClient from './client'
import { API_ENDPOINTS } from './config'
import type {
  RuleTemplate,
  RuleTemplateApiResponse,
  RuleTemplateQuery
} from '@/types/ruleTemplate'

class RuleTemplateService {
  async getTemplates(query?: RuleTemplateQuery): Promise<RuleTemplateApiResponse> {
    const response = await apiClient.get<RuleTemplateApiResponse>(
      API_ENDPOINTS.RULE_TEMPLATES,
      { params: query }
    )

    return response.data
  }
}

export const ruleTemplateService = new RuleTemplateService()
export type { RuleTemplate, RuleTemplateQuery }
