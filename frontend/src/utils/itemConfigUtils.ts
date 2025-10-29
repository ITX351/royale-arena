export interface RarityLevelConfig {
  internalName: string
  displayName: string
  prefix: string
  isAirdropped: boolean
}

export interface WeaponProperties {
  damage: number
  uses?: number
  votes: number
  aoeDamage?: number
  bleedDamage?: number
}

export interface WeaponConfig {
  internalName: string
  displayNames: string[]
  rarity?: string
  properties: WeaponProperties
}

export interface ArmorProperties {
  defense: number
  uses?: number
  votes: number
}

export interface ArmorConfig {
  internalName: string
  displayNames: string[]
  rarity?: string
  properties: ArmorProperties
}

export interface UtilityProperties {
  category: string
  votes?: number
  uses?: number
  targets?: number
  damage?: number
  usesNight?: number
}

export interface UtilityConfig {
  name: string
  internalName?: string
  rarity?: string
  properties: UtilityProperties
}

export interface ConsumableProperties {
  effectType: string
  effectValue: number
  cureBleed?: number
}

export interface ConsumableConfig {
  name: string
  internalName?: string
  rarity?: string
  properties: ConsumableProperties
}

export interface UpgraderConfig {
  internalName: string
  displayNames: string[]
  rarity?: string
}

export interface UpgradeRecipeConfig {
  result: string
  ingredients: string[]
}

export interface ItemsByCategoryConfig {
  weapons: WeaponConfig[]
  armors: ArmorConfig[]
  utilities: UtilityConfig[]
  consumables: ConsumableConfig[]
  upgraders: UpgraderConfig[]
}

export interface NormalizedItemsConfig {
  rarityLevels: RarityLevelConfig[]
  items: ItemsByCategoryConfig
  upgradeRecipes: Record<string, UpgradeRecipeConfig[]>
}

export interface ItemsConfigParseResult {
  config: NormalizedItemsConfig
  issues: string[]
}

const createEmptyItemsConfig = (): NormalizedItemsConfig => ({
  rarityLevels: [],
  items: {
    weapons: [],
    armors: [],
    utilities: [],
    consumables: [],
    upgraders: []
  },
  upgradeRecipes: {}
})

const toStringArray = (value: unknown): string[] => {
  if (Array.isArray(value)) {
    return value.filter((entry): entry is string => typeof entry === 'string')
  }

  if (typeof value === 'string' && value.length > 0) {
    return [value]
  }

  return []
}

const toOptionalNumber = (value: unknown): number | undefined =>
  typeof value === 'number' ? value : undefined

const toNumberWithDefault = (value: unknown, fallback: number): number =>
  typeof value === 'number' ? value : fallback

export function parseItemsConfig(rawItemsConfig: any): ItemsConfigParseResult {
  const issues: string[] = []
  const parsed = createEmptyItemsConfig()

  if (!rawItemsConfig || typeof rawItemsConfig !== 'object') {
    issues.push('items_config 应为对象')
    return { config: parsed, issues }
  }

  if (Array.isArray(rawItemsConfig.rarity_levels)) {
    parsed.rarityLevels = rawItemsConfig.rarity_levels.map((level: any) => ({
      internalName: typeof level?.internal_name === 'string' ? level.internal_name : '',
      displayName: typeof level?.display_name === 'string' ? level.display_name : '',
      prefix: typeof level?.prefix === 'string' ? level.prefix : '',
      isAirdropped:
        level?.is_airdropped !== undefined ? Boolean(level.is_airdropped) : true
    }))
  } else if (rawItemsConfig.rarity_levels !== undefined) {
    issues.push('items_config.rarity_levels 应为数组')
  }

  const itemsSection = rawItemsConfig.items
  if (!itemsSection || typeof itemsSection !== 'object') {
    if (itemsSection !== undefined) {
      issues.push('items_config.items 应为对象')
    }
  } else {
    const { weapons, armors, utilities, consumables, upgraders } = itemsSection

    if (Array.isArray(weapons)) {
      parsed.items.weapons = weapons.map((weapon: any) => {
        const weaponProperties = weapon?.properties ?? {}
        const properties: WeaponProperties = {
          damage: toNumberWithDefault(weaponProperties.damage, 0),
          votes: toNumberWithDefault(weaponProperties.votes, 0)
        }

        const uses = toOptionalNumber(weaponProperties.uses)
        if (uses !== undefined) {
          properties.uses = uses
        }

        const aoeDamage = toOptionalNumber(weaponProperties.aoe_damage)
        if (aoeDamage !== undefined) {
          properties.aoeDamage = aoeDamage
        }

        const bleedDamage = toOptionalNumber(weaponProperties.bleed_damage)
        if (bleedDamage !== undefined) {
          properties.bleedDamage = bleedDamage
        }

        const rarity = typeof weapon?.rarity === 'string' && weapon.rarity.length > 0
          ? weapon.rarity
          : undefined

        return {
          internalName: typeof weapon?.internal_name === 'string' ? weapon.internal_name : '',
          displayNames: toStringArray(weapon?.display_names),
          rarity,
          properties
        }
      })
    } else if (weapons !== undefined) {
      issues.push('items_config.items.weapons 应为数组')
    }

    if (Array.isArray(armors)) {
      parsed.items.armors = armors.map((armor: any) => {
        const armorProperties = armor?.properties ?? {}
        const properties: ArmorProperties = {
          defense: toNumberWithDefault(armorProperties.defense, 0),
          votes: toNumberWithDefault(armorProperties.votes, 0)
        }

        const uses = toOptionalNumber(armorProperties.uses)
        if (uses !== undefined) {
          properties.uses = uses
        }

        const rarity = typeof armor?.rarity === 'string' && armor.rarity.length > 0
          ? armor.rarity
          : undefined

        return {
          internalName: typeof armor?.internal_name === 'string' ? armor.internal_name : '',
          displayNames: toStringArray(armor?.display_names),
          rarity,
          properties
        }
      })
    } else if (armors !== undefined) {
      issues.push('items_config.items.armors 应为数组')
    }

    if (Array.isArray(utilities)) {
      parsed.items.utilities = utilities.map((utility: any) => {
        const utilityProperties = utility?.properties ?? {}
        const properties: UtilityProperties = {
          category: typeof utilityProperties.category === 'string' ? utilityProperties.category : ''
        }

        const votes = toOptionalNumber(utilityProperties.votes)
        if (votes !== undefined) {
          properties.votes = votes
        }

        const uses = toOptionalNumber(utilityProperties.uses)
        if (uses !== undefined) {
          properties.uses = uses
        }

        const targets = toOptionalNumber(utilityProperties.targets)
        if (targets !== undefined) {
          properties.targets = targets
        }

        const damage = toOptionalNumber(utilityProperties.damage)
        if (damage !== undefined) {
          properties.damage = damage
        }

        const usesNight = toOptionalNumber(utilityProperties.uses_night)
        if (usesNight !== undefined) {
          properties.usesNight = usesNight
        }

        const rarity = typeof utility?.rarity === 'string' && utility.rarity.length > 0
          ? utility.rarity
          : undefined

        return {
          name: typeof utility?.name === 'string' ? utility.name : '',
          internalName: typeof utility?.internal_name === 'string' ? utility.internal_name : undefined,
          rarity,
          properties
        }
      })
    } else if (utilities !== undefined) {
      issues.push('items_config.items.utilities 应为数组')
    }

    if (Array.isArray(consumables)) {
      parsed.items.consumables = consumables.map((consumable: any) => {
        const consumableProperties = consumable?.properties ?? {}
        const properties: ConsumableProperties = {
          effectType: typeof consumableProperties.effect_type === 'string' ? consumableProperties.effect_type : '',
          effectValue: toNumberWithDefault(consumableProperties.effect_value, 0)
        }

        const cureBleed = toOptionalNumber(consumableProperties.cure_bleed)
        if (cureBleed !== undefined) {
          properties.cureBleed = cureBleed
        }

        const rarity = typeof consumable?.rarity === 'string' && consumable.rarity.length > 0
          ? consumable.rarity
          : undefined

        return {
          name: typeof consumable?.name === 'string' ? consumable.name : '',
          internalName: typeof consumable?.internal_name === 'string' ? consumable.internal_name : undefined,
          rarity,
          properties
        }
      })
    } else if (consumables !== undefined) {
      issues.push('items_config.items.consumables 应为数组')
    }

    if (Array.isArray(upgraders)) {
      parsed.items.upgraders = upgraders.map((upgrader: any) => {
        const rarity = typeof upgrader?.rarity === 'string' && upgrader.rarity.length > 0
          ? upgrader.rarity
          : undefined

        return {
          internalName: typeof upgrader?.internal_name === 'string' ? upgrader.internal_name : '',
          displayNames: toStringArray(upgrader?.display_names),
          rarity
        }
      })
    } else if (upgraders !== undefined) {
      issues.push('items_config.items.upgraders 应为数组')
    }
  }

  if (rawItemsConfig.upgrade_recipes && typeof rawItemsConfig.upgrade_recipes === 'object') {
    const normalizedRecipes: Record<string, UpgradeRecipeConfig[]> = {}

    for (const [key, recipes] of Object.entries(rawItemsConfig.upgrade_recipes)) {
      if (Array.isArray(recipes)) {
        normalizedRecipes[key] = recipes.map((recipe: any) => ({
          result: typeof recipe?.result === 'string' ? recipe.result : '',
          ingredients: toStringArray(recipe?.ingredients)
        }))
      } else {
        issues.push(`升级配方 ${key} 应为数组`)
      }
    }

    parsed.upgradeRecipes = normalizedRecipes
  } else if (rawItemsConfig.upgrade_recipes !== undefined) {
    issues.push('items_config.upgrade_recipes 应为对象')
  }

  return { config: parsed, issues }
}

export function findDuplicateItemNames(itemsConfig: NormalizedItemsConfig): string[] {
  const seen = new Set<string>()
  const duplicates = new Set<string>()

  const trackName = (name?: string) => {
    if (!name) {
      return
    }
    if (seen.has(name)) {
      duplicates.add(name)
    } else {
      seen.add(name)
    }
  }

  for (const weapon of itemsConfig.items.weapons) {
    for (const displayName of weapon.displayNames) {
      trackName(displayName)
    }
  }

  for (const armor of itemsConfig.items.armors) {
    for (const displayName of armor.displayNames) {
      trackName(displayName)
    }
  }

  for (const utility of itemsConfig.items.utilities) {
    trackName(utility.name)
  }

  for (const consumable of itemsConfig.items.consumables) {
    trackName(consumable.name)
  }

  for (const upgrader of itemsConfig.items.upgraders) {
    for (const displayName of upgrader.displayNames) {
      trackName(displayName)
    }
  }

  return [...duplicates]
}
