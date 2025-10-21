export interface RuleTemplate {
  id: string
  template_name: string
  description: string | null
  is_active: boolean
  rules_config: unknown
  created_at: string
  updated_at: string
}

export interface RuleTemplateQuery {
  id?: string
  is_active?: boolean
  search?: string
}

export interface RuleTemplateApiResponse {
  success: boolean
  data: RuleTemplate[]
  message?: string
}
