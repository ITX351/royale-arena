// 物品解析工具模块 - 从游戏规则JSON中提取物品信息

import { findDuplicateItemNames, parseItemsConfig } from './itemConfigUtils'
import type {
  NormalizedItemsConfig,
  RarityLevelConfig,
  WeaponConfig as NormalizedWeaponConfig,
  ArmorConfig as NormalizedArmorConfig,
  UtilityConfig as NormalizedUtilityConfig,
  ConsumableConfig as NormalizedConsumableConfig,
  UpgraderConfig as NormalizedUpgraderConfig
} from './itemConfigUtils'
import type { DirectorGameData, Item } from '@/types/gameStateTypes'

export type RarityLevel = RarityLevelConfig
export type WeaponConfig = NormalizedWeaponConfig
export type ArmorConfig = NormalizedArmorConfig
export type UtilityConfig = NormalizedUtilityConfig
export type ConsumableConfig = NormalizedConsumableConfig
export type UpgraderConfig = NormalizedUpgraderConfig
export type ItemConfig = NormalizedItemsConfig

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
  utilities: string[]
  consumables: string[]
  upgraders: string[]
  // 稀有度配置
  rarityLevels: RarityLevelConfig[]
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
  private itemConfig: NormalizedItemsConfig
  private existingItems: Set<string> // 场上已存在的物品名称
  public readonly issues: string[]

  constructor(rulesJson: any, existingItems: string[] = []) {
    const itemsConfigRoot = rulesJson?.items_config
    if (!itemsConfigRoot) {
      throw new Error('规则JSON缺少 items_config 配置')
    }

    if (!itemsConfigRoot.items) {
      throw new Error('规则JSON缺少 items_config.items 配置')
    }

    const { config, issues } = parseItemsConfig(itemsConfigRoot)
    this.issues = issues

    const hasAnyItem =
      config.rarityLevels.length > 0 ||
      config.items.weapons.length > 0 ||
      config.items.armors.length > 0 ||
      config.items.utilities.length > 0 ||
      config.items.consumables.length > 0 ||
      config.items.upgraders.length > 0

    if (!hasAnyItem) {
      throw new Error('无法从规则JSON中解析物品配置，请检查 items_config.items 字段内容是否正确')
    }

    const duplicateNames = findDuplicateItemNames(config)
    if (duplicateNames.length > 0) {
      throw new Error(`规则JSON中发现重复的物品名称: ${duplicateNames.join(', ')}`)
    }

    this.itemConfig = config
    this.existingItems = new Set(existingItems)
  }

  /**
   * 解析所有物品信息
   */
  parseAllItems(): ParsedItemInfo {
    // 收集所有物品名称
    const allItems: string[] = []
    
    // 武器稀有度映射
    const weaponsByRarity: Record<string, string[]> = {}
    for (const weapon of this.itemConfig.items.weapons) {
      if (weapon.rarity && !weaponsByRarity[weapon.rarity]) {
        weaponsByRarity[weapon.rarity] = []
      }
      if (weapon.rarity) {
        weaponsByRarity[weapon.rarity].push(...weapon.displayNames)
      }
      allItems.push(...weapon.displayNames)
    }

    // 防具稀有度映射
    const armorsByRarity: Record<string, string[]> = {}
    for (const armor of this.itemConfig.items.armors) {
      if (armor.rarity && !armorsByRarity[armor.rarity]) {
        armorsByRarity[armor.rarity] = []
      }
      if (armor.rarity) {
        armorsByRarity[armor.rarity].push(...armor.displayNames)
      }
      allItems.push(...armor.displayNames)
    }

    // 功能道具
    const utilities = this.itemConfig.items.utilities.map(item => item.name)
    allItems.push(...utilities)

    // 消耗品
    const consumables = this.itemConfig.items.consumables.map(item => item.name)
    allItems.push(...consumables)

    // 升级器
    const upgraders = this.itemConfig.items.upgraders.flatMap(upgrader => upgrader.displayNames)
    allItems.push(...upgraders)

    return {
      allItems,
      rarityItems: {
        weapons: weaponsByRarity,
        armors: armorsByRarity
      },
      utilities,
      consumables,
      upgraders,
      rarityLevels: this.itemConfig.rarityLevels
    }
  }

  /**
   * 获取批量空投的武器和防具稀有度选项（考虑场上已存在的物品）
   */
  getBatchAirdropRarityOptions(): {
    weapons: RarityOption[]
    armors: RarityOption[]
  } {
    const weaponOptions: RarityOption[] = []
    const armorOptions: RarityOption[] = []
    const parsedItems = this.parseAllItems()

    // 处理武器稀有度
    for (const rarity of this.itemConfig.rarityLevels) {
      const weaponNames = parsedItems.rarityItems.weapons[rarity.internalName] || []
      const availableWeapons = weaponNames.filter(name => !this.existingItems.has(name))
      
      if (weaponNames.length > 0) {
        weaponOptions.push({
          rarityKey: `weapon_${rarity.internalName}`,
          displayName: `${rarity.displayName}武器(上限${availableWeapons.length})`,
          itemType: 'weapon',
          availableCount: availableWeapons.length,
          maxCount: weaponNames.length
        })
      }
    }

    // 处理防具稀有度
    for (const rarity of this.itemConfig.rarityLevels) {
      const armorNames = parsedItems.rarityItems.armors[rarity.internalName] || []
      const availableArmors = armorNames.filter(name => !this.existingItems.has(name))
      
      if (armorNames.length > 0) {
        armorOptions.push({
          rarityKey: `armor_${rarity.internalName}`,
          displayName: `${rarity.displayName}防具(上限${availableArmors.length})`,
          itemType: 'armor',
          availableCount: availableArmors.length,
          maxCount: armorNames.length
        })
      }
    }

    return {
      weapons: weaponOptions,
      armors: armorOptions
    }
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
   * 获取可空投的物品列表（全部物品减去场上已存在的物品）
   */
  getAvailableAirdropItems(): string[] {
    const allItems = this.parseAllItems()
    return allItems.allItems.filter(name => !this.existingItems.has(name))
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

/**
 * 从游戏状态中提取场上已存在的物品名称
 */
/**
 * 判断一个物品对象是否属于武器或防具
 * 返回 true 表示该物品的 item_type.type 为 'weapon' 或 'armor'
 */
export function isWeaponOrArmor(item: Item | any): boolean {
  if (!item || typeof item !== 'object') return false
  const itemType = item.item_type
  if (!itemType || typeof itemType !== 'object') return false
  const t = itemType.type
  return t === 'weapon' || t === 'armor'
}

/**
 * 从导演视角的游戏数据中提取场上已存在的武器和防具名称
 * - 地点与背包中的物品会先通过 isWeaponOrArmor 判断类型
 * - 已装备的武器/护甲字段 (equipped_weapon / equipped_armor) 直接加入（只要存在 name）
 */
export function extractExistingWeaponsAndArmorsFromGameState(gameData: DirectorGameData | null): string[] {
  const existingItems: string[] = []

  // 从地点中提取物品（仅武器/防具）
  if (gameData && gameData.places) {
    for (const place of Object.values(gameData.places)) {
      if (place.items && Array.isArray(place.items)) {
        for (const item of place.items) {
          if (item && item.name && isWeaponOrArmor(item)) {
            existingItems.push(item.name)
          }
        }
      }
    }
  }

  // 从玩家身上提取物品（包括装备和背包） - 装备项直接加入
  if (gameData && gameData.players) {
    for (const player of Object.values(gameData.players)) {
      // 添加已装备的武器
      if (player.equipped_weapon && player.equipped_weapon.name) {
        existingItems.push(player.equipped_weapon.name)
      }

      // 添加已装备的防具
      if (player.equipped_armor && player.equipped_armor.name) {
        existingItems.push(player.equipped_armor.name)
      }

      // 添加背包中的物品（仅武器/防具）
      if (player.inventory && Array.isArray(player.inventory)) {
        for (const item of player.inventory) {
          if (item && item.name && isWeaponOrArmor(item)) {
            existingItems.push(item.name)
          }
        }
      }
    }
  }

  return existingItems
}