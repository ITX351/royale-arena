// 物品解析工具模块 - 从游戏规则JSON中提取物品信息

// 稀有度配置接口
export interface RarityLevel {
  internal_name: string
  display_name: string
  prefix: string
  is_airdropped: boolean
}

// 武器配置接口
export interface WeaponConfig {
  internal_name: string
  display_names: string[]
  rarity: string
  properties: Record<string, any>
}

// 防具配置接口
export interface ArmorConfig {
  internal_name: string
  display_names: string[]
  rarity: string
  properties: Record<string, any>
}

// 其他道具配置接口
export interface OtherItemConfig {
  name: string
  category: string
  properties: Record<string, any>
}

// 消耗品配置接口
export interface ConsumableConfig {
  name: string
  effect_type: string
  effect_value: number
  cure_bleed?: boolean
}

// 升级器配置接口
export interface UpgraderConfig {
  internal_name: string
  display_names: string[]
  rarity: string
}

// 物品配置接口
export interface ItemConfig {
  rarity_levels: RarityLevel[]
  weapons: WeaponConfig[]
  armors: ArmorConfig[]
  other_items: OtherItemConfig[]
  consumables: ConsumableConfig[]
  upgraders: UpgraderConfig[]
}

// 解析后的物品信息接口
export interface ParsedItemInfo {
  // 全量物品列表（用于单次空投）
  allItems: string[]
  // 稀有度物品映射（用于批量空投）
  rarityItems: {
    weapons: Record<string, string[]>  // 稀有度 -> 武器名称列表
    armors: Record<string, string[]>   // 稀有度 -> 防具名称列表
  }
  // 其他物品类型
  otherItems: string[]
  consumables: string[]
  upgraders: string[]
  // 稀有度配置
  rarityLevels: RarityLevel[]
}

// 批量空投可选择的稀有度类型（带数量上限）
export interface RarityOption {
  rarityKey: string      // 稀有度键
  displayName: string    // 显示名称
  itemType: 'weapon' | 'armor'  // 物品类型
  availableCount: number // 可用数量（场上未出现的）
  maxCount: number       // 最大数量（该稀有度所有物品）
}

/**
 * 物品解析器类
 */
export class ItemParser {
  private itemConfig: ItemConfig
  private existingItems: Set<string> // 场上已存在的物品名称

  constructor(rulesJson: any, existingItems: string[] = []) {
    this.itemConfig = rulesJson.items
    this.existingItems = new Set(existingItems)
    
    // 验证规则JSON中是否有重复物品名称
    this.validateUniqueItemNames()
  }

  /**
   * 验证规则JSON中物品名称的唯一性
   */
  private validateUniqueItemNames(): void {
    const allItemNames = new Set<string>()
    const duplicateNames: string[] = []
    
    // 检查武器名称
    for (const weapon of this.itemConfig.weapons) {
      for (const displayName of weapon.display_names) {
        if (allItemNames.has(displayName)) {
          duplicateNames.push(displayName)
        } else {
          allItemNames.add(displayName)
        }
      }
    }
    
    // 检查防具名称
    for (const armor of this.itemConfig.armors) {
      for (const displayName of armor.display_names) {
        if (allItemNames.has(displayName)) {
          duplicateNames.push(displayName)
        } else {
          allItemNames.add(displayName)
        }
      }
    }
    
    // 检查其他道具名称
    for (const otherItem of this.itemConfig.other_items) {
      if (allItemNames.has(otherItem.name)) {
        duplicateNames.push(otherItem.name)
      } else {
        allItemNames.add(otherItem.name)
      }
    }
    
    // 检查消耗品名称
    for (const consumable of this.itemConfig.consumables) {
      if (allItemNames.has(consumable.name)) {
        duplicateNames.push(consumable.name)
      } else {
        allItemNames.add(consumable.name)
      }
    }
    
    // 检查升级器名称
    for (const upgrader of this.itemConfig.upgraders) {
      for (const displayName of upgrader.display_names) {
        if (allItemNames.has(displayName)) {
          duplicateNames.push(displayName)
        } else {
          allItemNames.add(displayName)
        }
      }
    }
    
    // 如果发现重复名称，抛出错误
    if (duplicateNames.length > 0) {
      const uniqueDuplicates = [...new Set(duplicateNames)]
      throw new Error(`规则JSON中发现重复的物品名称: ${uniqueDuplicates.join(', ')}`)
    }
  }

  /**
   * 解析所有物品信息
   */
  parseAllItems(): ParsedItemInfo {
    // 收集所有物品名称
    const allItems: string[] = []
    
    // 武器稀有度映射
    const weaponsByRarity: Record<string, string[]> = {}
    for (const weapon of this.itemConfig.weapons) {
      if (!weaponsByRarity[weapon.rarity]) {
        weaponsByRarity[weapon.rarity] = []
      }
      weaponsByRarity[weapon.rarity].push(...weapon.display_names)
      allItems.push(...weapon.display_names)
    }

    // 防具稀有度映射
    const armorsByRarity: Record<string, string[]> = {}
    for (const armor of this.itemConfig.armors) {
      if (!armorsByRarity[armor.rarity]) {
        armorsByRarity[armor.rarity] = []
      }
      armorsByRarity[armor.rarity].push(...armor.display_names)
      allItems.push(...armor.display_names)
    }

    // 其他道具
    const otherItems = this.itemConfig.other_items.map(item => item.name)
    allItems.push(...otherItems)

    // 消耗品
    const consumables = this.itemConfig.consumables.map(item => item.name)
    allItems.push(...consumables)

    // 升级器
    const upgraders = this.itemConfig.upgraders.flatMap(upgrader => upgrader.display_names)
    allItems.push(...upgraders)

    return {
      allItems,
      rarityItems: {
        weapons: weaponsByRarity,
        armors: armorsByRarity
      },
      otherItems,
      consumables,
      upgraders,
      rarityLevels: this.itemConfig.rarity_levels
    }
  }

  /**
   * 获取批量空投的稀有度选项（考虑场上已存在的物品）
   */
  getBatchAirdropRarityOptions(): RarityOption[] {
    const options: RarityOption[] = []
    const parsedItems = this.parseAllItems()

    // 处理武器稀有度
    for (const rarity of this.itemConfig.rarity_levels) {
      const weaponNames = parsedItems.rarityItems.weapons[rarity.internal_name] || []
      const availableWeapons = weaponNames.filter(name => !this.existingItems.has(name))
      
      if (weaponNames.length > 0) {
        options.push({
          rarityKey: `weapon_${rarity.internal_name}`,
          displayName: `${rarity.display_name}武器(上限${availableWeapons.length})`,
          itemType: 'weapon',
          availableCount: availableWeapons.length,
          maxCount: weaponNames.length
        })
      }
    }

    // 处理防具稀有度
    for (const rarity of this.itemConfig.rarity_levels) {
      const armorNames = parsedItems.rarityItems.armors[rarity.internal_name] || []
      const availableArmors = armorNames.filter(name => !this.existingItems.has(name))
      
      if (armorNames.length > 0) {
        options.push({
          rarityKey: `armor_${rarity.internal_name}`,
          displayName: `${rarity.display_name}防具(上限${availableArmors.length})`,
          itemType: 'armor',
          availableCount: availableArmors.length,
          maxCount: armorNames.length
        })
      }
    }

    return options
  }

  /**
   * 根据稀有度和数量随机挑选物品名称
   */
  pickItemsByRarity(rarityKey: string, count: number): { 
    selectedItems: string[], 
    isInsufficient: boolean 
  } {
    const parsedItems = this.parseAllItems()
    
    // 解析稀有度键
    const [itemType, rarity] = rarityKey.split('_', 2)
    
    let availableItems: string[] = []
    
    if (itemType === 'weapon') {
      const allWeapons = parsedItems.rarityItems.weapons[rarity] || []
      availableItems = allWeapons.filter(name => !this.existingItems.has(name))
    } else if (itemType === 'armor') {
      const allArmors = parsedItems.rarityItems.armors[rarity] || []
      availableItems = allArmors.filter(name => !this.existingItems.has(name))
    }

    // 检查是否不足
    const isInsufficient = availableItems.length < count
    
    // 随机挑选
    const selectedItems: string[] = []
    const itemsCopy = [...availableItems]
    
    for (let i = 0; i < Math.min(count, availableItems.length); i++) {
      const randomIndex = Math.floor(Math.random() * itemsCopy.length)
      selectedItems.push(itemsCopy.splice(randomIndex, 1)[0])
    }

    return { selectedItems, isInsufficient }
  }

  /**
   * 获取可用地点列表（排除已摧毁的地点）
   */
  getAvailablePlaces(allPlaces: any[]): string[] {
    return allPlaces
      .filter(place => !place.is_destroyed)
      .map(place => place.name)
  }

  /**
   * 为物品列表随机分配地点
   */
  randomAssignPlaces(itemNames: string[], availablePlaces: string[]): Array<{
    item_name: string
    place_name: string
  }> {
    const result: Array<{ item_name: string, place_name: string }> = []
    
    for (const itemName of itemNames) {
      const randomPlace = availablePlaces[Math.floor(Math.random() * availablePlaces.length)]
      result.push({
        item_name: itemName,
        place_name: randomPlace
      })
    }
    
    return result
  }

  /**
   * 更新场上已存在的物品列表
   */
  updateExistingItems(existingItems: string[]) {
    this.existingItems = new Set(existingItems)
  }
}

/**
 * 创建物品解析器的工厂函数
 */
export function createItemParser(rulesJson: any, existingItems: string[] = []): ItemParser {
  return new ItemParser(rulesJson, existingItems)
}

import type { DirectorGameData, DirectorPlace } from '@/types/gameStateTypes'

/**
 * 从游戏状态中提取场上已存在的物品名称
 */
export function extractExistingItemsFromGameState(gameData: DirectorGameData | null): string[] {
  const existingItems: string[] = []
  
  if (gameData && gameData.places) {
    for (const place of Object.values(gameData.places)) {
      if (place.items && Array.isArray(place.items)) {
        for (const item of place.items) {
          if (item.name) {
            existingItems.push(item.name)
          }
        }
      }
    }
  }
  
  return existingItems
}