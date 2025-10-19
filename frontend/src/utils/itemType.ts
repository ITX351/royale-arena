import type { ItemCategory } from '@/types/gameStateTypes'

const LABEL_MAP: Record<ItemCategory, string> = {
  weapon: '武器',
  armor: '防具',
  consumable: '消耗品',
  utility: '工具',
  upgrader: '升级器',
}

const TAG_TYPE_MAP: Record<ItemCategory, string> = {
  weapon: 'danger',
  armor: 'primary',
  consumable: 'success',
  utility: 'warning',
  upgrader: 'info',
}

export const getItemTypeLabel = (itemType?: ItemCategory | null): string => {
  if (!itemType) {
    return '未知'
  }
  return LABEL_MAP[itemType] ?? '未知'
}

export const getItemTypeTagType = (itemType?: ItemCategory | null): string => {
  if (!itemType) {
    return 'info'
  }
  return TAG_TYPE_MAP[itemType] ?? 'info'
}
