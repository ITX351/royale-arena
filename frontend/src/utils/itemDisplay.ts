import type { Item } from '@/types/gameStateTypes'

export interface ItemDisplayProperty {
  label: string
  value: string | number
}

export const formatItemProperty = (property: ItemDisplayProperty): string => {
  if (!property.label) {
    return String(property.value)
  }
  return `${property.label}: ${property.value}`
}

const RARITY_LABELS: Record<string, string> = {
  common: '普通',
  rare: '稀有',
  epic: '史诗',
  legendary: '传说',
}

const EFFECT_TYPE_LABELS: Record<string, string> = {
  heal: '治疗',
  strength: '回体',
}

const UTILITY_TYPE_LABELS: Record<string, string> = {
  utility_locator: '定位',
  utility_revealer: '侦察',
  utility_seer: '洞察',
  trap: '陷阱',
}

const UPGRADE_TYPE_LABELS: Record<string, string> = {
  natural: '自然升级',
  natural_upgrader: '自然升级',
  artificial: '人造升级',
  artificial_upgrader: '人造升级',
}

export const getItemDisplayProperties = (item: Item): ItemDisplayProperty[] => {
  const entries: ItemDisplayProperty[] = []
  const itemType = item.item_type?.type

  if (!itemType) {
    return entries
  }

  if (item.rarity) {
    entries.push({
      label: '', // 稀有度
      value: RARITY_LABELS[item.rarity] ?? item.rarity,
    })
  }

  const properties = item.item_type?.properties ?? {}

  switch (itemType) {
    case 'weapon':
      if (properties.damage != null) {
        entries.push({ label: '伤害', value: properties.damage })
      }
      if (properties.aoe_damage != null) {
        entries.push({ label: '溅射', value: properties.aoe_damage })
      }
      if (properties.bleed_damage != null) {
        entries.push({ label: '流血', value: properties.bleed_damage })
      }
      break
    case 'armor':
      if (properties.defense != null) {
        entries.push({ label: '防御', value: properties.defense })
      }
      break
    case 'consumable':
      if (properties.effect_type) {
        entries.push({
          label: '',
          value: EFFECT_TYPE_LABELS[properties.effect_type] ?? properties.effect_type,
        })
      }
      if (properties.effect_value != null) {
        entries.push({ label: '效果', value: properties.effect_value })
      }
      if (properties.cure_bleed != null && properties.cure_bleed > 0) {
        let cureDescription: string | number = properties.cure_bleed
        if (properties.cure_bleed === 1) {
          cureDescription = '抵消流血'
        } else if (properties.cure_bleed === 2) {
          cureDescription = '痊愈流血'
        }
        entries.push({ label: '', value: cureDescription }) // 止血描述
      }
      break
    case 'utility':
      if (properties.utility_type) {
        entries.push({
          label: '功能',
          value: UTILITY_TYPE_LABELS[properties.utility_type] ?? properties.utility_type,
        })
      }
      if (properties.targets != null) {
        entries.push({ label: '目标', value: properties.targets })
      }
      if (properties.damage != null) {
        entries.push({ label: '伤害', value: properties.damage })
      }
      break
    case 'upgrader':
      if (properties.upgrader_type) {
        entries.push({
          label: '',
          value: UPGRADE_TYPE_LABELS[properties.upgrader_type] ?? properties.upgrader_type,
        })
      }
      break
  }

  if (properties.uses != null) {
    entries.push({ label: '次数', value: properties.uses })
  }

  if (properties.votes != null) {
    entries.push({ label: '票数', value: properties.votes })
  }

  return entries
}
