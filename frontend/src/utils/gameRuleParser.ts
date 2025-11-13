import { findDuplicateItemNames, parseItemsConfig } from './itemConfigUtils'
import type { NormalizedItemsConfig } from './itemConfigUtils'

const ALLOWED_DEATH_ITEM_DISPOSITIONS = [
	'killer_takes_loot',
	'drop_to_ground',
	'vanish_completely'
] as const

type DeathItemDisposition = typeof ALLOWED_DEATH_ITEM_DISPOSITIONS[number]

export interface ParsedGameRules {
	map: {
		places: string[]
		safePlaces: string[]
	}
	player: {
		maxLife: number
		maxStrength: number
		dailyLifeRecovery: number
		dailyStrengthRecovery: number
		searchCooldown: number
		maxBackpackItems: number
		unarmedDamage: number
	}
	actionCosts: {
		move: number
		search: number
		pick: number
		attack: number
		equip: number
		use: number
		throw: number
		deliver: number
	}
	restMode: {
		lifeRecovery: number
		strengthRecovery: number
		maxMoves: number
	}
	deathItemDisposition: DeathItemDisposition
	teammateBehavior: number
	parsedTeammateBehaviors: {
		noHarm: boolean
		noSearch: boolean
		canViewStatus: boolean
		canTransferItems: boolean
	}
	itemsConfig: NormalizedItemsConfig
	displayNames: {
		playerMaxLife: string
		playerMaxStrength: string
		playerDailyLifeRecovery: string
		playerDailyStrengthRecovery: string
		playerSearchCooldown: string
		actionMove: string
		actionSearch: string
		actionPick: string
		actionAttack: string
		actionEquip: string
		actionUse: string
		actionThrow: string
		actionDeliver: string
		playerUnarmedDamage: string
		restLifeRecovery: string
		restMaxMoves: string
	}
	parsingIssues: string[]
	missingSections: string[]
}

export interface GameRuleValidationResult {
	isValid: boolean
	errors: string[]
	missingSections: string[]
}

export class GameRuleParserError extends Error {
	public readonly errors: string[]
	public readonly missingSections: string[]

	constructor(message: string, errors: string[], missingSections: string[]) {
		super(message)
		this.name = 'GameRuleParserError'
		this.errors = errors
		this.missingSections = missingSections
	}
}

export class GameRuleParser {
	parse(rulesConfig: unknown): ParsedGameRules {
		const validation = this.validate(rulesConfig)
		if (!validation.isValid) {
			throw new GameRuleParserError('规则JSON验证失败', validation.errors, validation.missingSections)
		}

		const config = rulesConfig as any

		const { config: parsedItemsConfig, issues: itemIssues } = parseItemsConfig(config.items_config)
		const duplicateNames = findDuplicateItemNames(parsedItemsConfig)

		const parsingIssues: string[] = [...itemIssues]
		if (duplicateNames.length > 0) {
			parsingIssues.push(`规则JSON中发现重复的物品名称: ${duplicateNames.join(', ')}`)
		}

		return {
			map: {
				places: [...config.map.places],
				safePlaces: [...config.map.safe_places]
			},
			player: {
				maxLife: config.player.max_life,
				maxStrength: config.player.max_strength,
				dailyLifeRecovery: config.player.daily_life_recovery,
				dailyStrengthRecovery: config.player.daily_strength_recovery,
				searchCooldown: config.player.search_cooldown,
				maxBackpackItems: config.player.max_backpack_items,
				unarmedDamage: config.player.unarmed_damage
			},
			actionCosts: {
				move: config.action_costs.move,
				search: config.action_costs.search,
				pick: config.action_costs.pick,
				attack: config.action_costs.attack,
				equip: config.action_costs.equip,
				use: config.action_costs.use,
				throw: config.action_costs.throw,
				deliver: config.action_costs.deliver
			},
			restMode: {
				lifeRecovery: config.rest_mode.life_recovery,
				strengthRecovery: config.rest_mode.strength_recovery,
				maxMoves: config.rest_mode.max_moves
			},
			deathItemDisposition: config.death_item_disposition,
			teammateBehavior: config.teammate_behavior,
			parsedTeammateBehaviors: this.parseTeammateBehavior(config.teammate_behavior),
			itemsConfig: parsedItemsConfig,
			displayNames: {
				playerMaxLife: config.display_names.player_max_life,
				playerMaxStrength: config.display_names.player_max_strength,
				playerDailyLifeRecovery: config.display_names.player_daily_life_recovery,
				playerDailyStrengthRecovery: config.display_names.player_daily_strength_recovery,
				playerSearchCooldown: config.display_names.player_search_cooldown,
				actionMove: config.display_names.action_move,
				actionSearch: config.display_names.action_search,
				actionPick: config.display_names.action_pick,
				actionAttack: config.display_names.action_attack,
				actionEquip: config.display_names.action_equip,
				actionUse: config.display_names.action_use,
				actionThrow: config.display_names.action_throw,
				actionDeliver: config.display_names.action_deliver,
				playerUnarmedDamage: config.display_names.player_unarmed_damage,
				restLifeRecovery: config.display_names.rest_life_recovery,
				restMaxMoves: config.display_names.rest_max_moves
			},
			parsingIssues,
			missingSections: []
		}
	}

	validate(rulesConfig: unknown): GameRuleValidationResult {
		const errors: string[] = []
		const missingSections: string[] = []

		if (!rulesConfig || typeof rulesConfig !== 'object') {
			errors.push('规则配置必须是对象')
			missingSections.push('root')
			return { isValid: false, errors, missingSections }
		}

		const config = rulesConfig as Record<string, any>
		const requiredFields = [
			'map',
			'player',
			'action_costs',
			'rest_mode',
			'teammate_behavior',
			'items_config',
			'display_names',
			'death_item_disposition'
		]

		for (const field of requiredFields) {
			if (!Object.prototype.hasOwnProperty.call(config, field)) {
				missingSections.push(field)
				errors.push(`缺少必需字段: ${field}`)
			}
		}

		if (config.map !== undefined) {
			if (!config.map || typeof config.map !== 'object') {
				errors.push('map 必须是对象')
			} else {
				if (!Object.prototype.hasOwnProperty.call(config.map, 'places')) {
					errors.push('map.places 为必填字段')
				} else if (!Array.isArray(config.map.places)) {
					errors.push('地图地点必须是数组')
				} else {
					const places = config.map.places as unknown[]
					if (places.length === 0) {
						errors.push('地图地点列表不能为空')
					}
					const invalidEntries = places.filter(place => typeof place !== 'string' || place.trim().length === 0)
					if (invalidEntries.length > 0) {
						errors.push('地图地点必须为非空字符串')
					}
					const duplicates = this.findDuplicates(places as string[])
					if (duplicates.length > 0) {
						errors.push(`地图地点列表存在重复项: ${duplicates.join(', ')}`)
					}
				}

				if (!Object.prototype.hasOwnProperty.call(config.map, 'safe_places')) {
					errors.push('map.safe_places 为必填字段')
				} else if (!Array.isArray(config.map.safe_places)) {
					errors.push('安全地点必须是数组')
				} else {
					const safePlaces = config.map.safe_places as unknown[]
					const invalidSafeEntries = safePlaces.filter(place => typeof place !== 'string' || place.trim().length === 0)
					if (invalidSafeEntries.length > 0) {
						errors.push('安全地点必须为非空字符串')
					}
					const duplicates = this.findDuplicates(safePlaces as string[])
					if (duplicates.length > 0) {
						errors.push(`安全地点列表存在重复项: ${duplicates.join(', ')}`)
					}
					if (Array.isArray(config.map.places)) {
						const missingSafePlaces = this.findMissingSafePlaces(
							safePlaces as string[],
							config.map.places as string[]
						)
						if (missingSafePlaces.length > 0) {
							errors.push(`安全地点列表包含未在地点名单中的地点: ${missingSafePlaces.join(', ')}`)
						}
					}
				}
			}
		}

		if (config.player !== undefined) {
			if (!config.player || typeof config.player !== 'object') {
				errors.push('player 必须是对象')
			} else {
				const playerFields: Array<[string, string]> = [
					['max_life', '玩家最大生命值'],
					['max_strength', '玩家最大体力值'],
					['daily_life_recovery', '玩家每日生命恢复值'],
					['daily_strength_recovery', '玩家每日体力恢复值'],
					['search_cooldown', '玩家搜索冷却时间'],
					['max_backpack_items', '玩家背包最大物品数'],
					['unarmed_damage', '玩家挥拳伤害']
				]

				for (const [field, label] of playerFields) {
					if (!Object.prototype.hasOwnProperty.call(config.player, field)) {
						errors.push(`缺少 player.${field} 字段`)
						continue
					}
					const value = config.player[field]
					if (typeof value !== 'number' || !Number.isFinite(value)) {
						errors.push(`${label}必须是数字`)
					}
				}
			}
		}

		if (config.action_costs !== undefined) {
			if (!config.action_costs || typeof config.action_costs !== 'object') {
				errors.push('action_costs 必须是对象')
			} else {
				const actionFields: Array<[string, string]> = [
					['move', '移动体力消耗'],
					['search', '搜索体力消耗'],
					['pick', '拾取体力消耗'],
					['attack', '攻击体力消耗'],
					['equip', '装备体力消耗'],
					['use', '使用体力消耗'],
					['throw', '丢弃体力消耗'],
					['deliver', '传音体力消耗']
				]

				for (const [field, label] of actionFields) {
					if (!Object.prototype.hasOwnProperty.call(config.action_costs, field)) {
						errors.push(`缺少 action_costs.${field} 字段`)
						continue
					}
					const value = config.action_costs[field]
					if (typeof value !== 'number' || !Number.isFinite(value)) {
						errors.push(`${label}必须是数字`)
					}
				}
			}
		}

		if (config.rest_mode !== undefined) {
			if (!config.rest_mode || typeof config.rest_mode !== 'object') {
				errors.push('rest_mode 必须是对象')
			} else {
				const restFields: Array<[string, string]> = [
					['life_recovery', '静养生命恢复值'],
					['strength_recovery', '静养体力恢复值'],
					['max_moves', '静养最大移动次数']
				]

				for (const [field, label] of restFields) {
					if (!Object.prototype.hasOwnProperty.call(config.rest_mode, field)) {
						errors.push(`缺少 rest_mode.${field} 字段`)
						continue
					}
					const value = config.rest_mode[field]
					if (typeof value !== 'number' || !Number.isFinite(value)) {
						errors.push(`${label}必须是数字`)
					}
				}
			}
		}

		if (config.death_item_disposition !== undefined) {
			if (typeof config.death_item_disposition !== 'string') {
				errors.push('死亡后物品去向必须是字符串')
			} else if (!ALLOWED_DEATH_ITEM_DISPOSITIONS.includes(config.death_item_disposition as DeathItemDisposition)) {
				errors.push('死亡后物品去向值无效，必须是: killer_takes_loot, drop_to_ground, vanish_completely 之一')
			}
		}

		if (!Object.prototype.hasOwnProperty.call(config, 'teammate_behavior')) {
			// 已在 requiredFields 中记录
		} else if (typeof config.teammate_behavior !== 'number' || !Number.isFinite(config.teammate_behavior)) {
			errors.push('teammate_behavior 必须是数字')
		} else if (!Number.isInteger(config.teammate_behavior) || config.teammate_behavior < 0) {
			errors.push('teammate_behavior 必须是非负整数')
		}

		if (config.items_config !== undefined) {
			if (!config.items_config || typeof config.items_config !== 'object') {
				errors.push('items_config 必须是对象')
			} else {
				const itemsConfig = config.items_config

				if (!Object.prototype.hasOwnProperty.call(itemsConfig, 'rarity_levels')) {
					errors.push('items_config.rarity_levels 为必填字段')
				} else if (!Array.isArray(itemsConfig.rarity_levels)) {
					errors.push('items_config.rarity_levels 必须是数组')
				}

				if (!Object.prototype.hasOwnProperty.call(itemsConfig, 'items')) {
					errors.push('items_config.items 为必填字段')
				} else if (!itemsConfig.items || typeof itemsConfig.items !== 'object') {
					errors.push('items_config.items 必须存在且为对象')
				} else {
					const categories = itemsConfig.items

					if (!Array.isArray(categories.weapons)) {
						errors.push('items_config.items.weapons 必须是数组')
					} else {
						categories.weapons.forEach((weapon: any, index: number) => {
							if (!weapon || typeof weapon !== 'object') {
								errors.push(`武器[${index}]配置必须是对象`)
								return
							}
							if (!weapon.internal_name || typeof weapon.internal_name !== 'string' || weapon.internal_name.trim().length === 0) {
								errors.push(`武器[${index}]缺少内部名称`)
							}
							if (!Array.isArray(weapon.display_names) || weapon.display_names.length === 0) {
								errors.push(`武器[${index}]显示名称必须是非空数组`)
							}
							if (!weapon.properties || typeof weapon.properties !== 'object') {
								errors.push(`武器[${index}]缺少属性配置`)
							} else {
								const properties = weapon.properties
								if (typeof properties.damage !== 'number' || !Number.isFinite(properties.damage)) {
									errors.push(`武器[${index}]伤害值必须是数字`)
								}
								if (properties.uses !== undefined && (typeof properties.uses !== 'number' || !Number.isFinite(properties.uses))) {
									errors.push(`武器[${index}]使用次数必须是数字`)
								}
								if (typeof properties.votes !== 'number' || !Number.isFinite(properties.votes)) {
									errors.push(`武器[${index}]票数必须是数字`)
								}
								if (properties.aoe_damage !== undefined && (typeof properties.aoe_damage !== 'number' || !Number.isFinite(properties.aoe_damage))) {
									errors.push(`武器[${index}]范围伤害必须是数字`)
								}
								if (properties.bleed_damage !== undefined && (typeof properties.bleed_damage !== 'number' || !Number.isFinite(properties.bleed_damage))) {
									errors.push(`武器[${index}]流血伤害必须是数字`)
								}
							}
						})
					}

					if (!Array.isArray(categories.armors)) {
						errors.push('items_config.items.armors 必须是数组')
					} else {
						categories.armors.forEach((armor: any, index: number) => {
							if (!armor || typeof armor !== 'object') {
								errors.push(`防具[${index}]配置必须是对象`)
								return
							}
							if (!armor.internal_name || typeof armor.internal_name !== 'string' || armor.internal_name.trim().length === 0) {
								errors.push(`防具[${index}]缺少内部名称`)
							}
							if (!Array.isArray(armor.display_names) || armor.display_names.length === 0) {
								errors.push(`防具[${index}]显示名称必须是非空数组`)
							}
							if (!armor.properties || typeof armor.properties !== 'object') {
								errors.push(`防具[${index}]缺少属性配置`)
							} else {
								const properties = armor.properties
								if (typeof properties.defense !== 'number' || !Number.isFinite(properties.defense)) {
									errors.push(`防具[${index}]防御值必须是数字`)
								}
								if (properties.uses !== undefined && (typeof properties.uses !== 'number' || !Number.isFinite(properties.uses))) {
									errors.push(`防具[${index}]使用次数必须是数字`)
								}
								if (typeof properties.votes !== 'number' || !Number.isFinite(properties.votes)) {
									errors.push(`防具[${index}]票数必须是数字`)
								}
							}
						})
					}

					if (!Array.isArray(categories.utilities)) {
						errors.push('items_config.items.utilities 必须是数组')
					} else {
						categories.utilities.forEach((utility: any, index: number) => {
							if (!utility || typeof utility !== 'object') {
								errors.push(`功能物品[${index}]配置必须是对象`)
								return
							}
							if (!utility.name || typeof utility.name !== 'string' || utility.name.trim().length === 0) {
								errors.push(`功能物品[${index}]缺少名称`)
							}
							if (!utility.properties || typeof utility.properties !== 'object') {
								errors.push(`功能物品[${index}]缺少属性配置`)
							} else {
								const properties = utility.properties
								if (!properties.category || typeof properties.category !== 'string' || properties.category.trim().length === 0) {
									errors.push(`功能物品[${index}]缺少分类`)
								}
								if (properties.votes !== undefined && (typeof properties.votes !== 'number' || !Number.isFinite(properties.votes))) {
									errors.push(`功能物品[${index}]票数必须是数字`)
								}
								if (properties.uses !== undefined && (typeof properties.uses !== 'number' || !Number.isFinite(properties.uses))) {
									errors.push(`功能物品[${index}]使用次数必须是数字`)
								}
								if (properties.targets !== undefined && (typeof properties.targets !== 'number' || !Number.isFinite(properties.targets))) {
									errors.push(`功能物品[${index}]目标数量必须是数字`)
								}
								if (properties.damage !== undefined && (typeof properties.damage !== 'number' || !Number.isFinite(properties.damage))) {
									errors.push(`功能物品[${index}]伤害值必须是数字`)
								}
								if (properties.uses_night !== undefined && (typeof properties.uses_night !== 'number' || !Number.isFinite(properties.uses_night))) {
									errors.push(`功能物品[${index}]夜间使用次数必须是数字`)
								}
							}
						})
					}

					if (!Array.isArray(categories.consumables)) {
						errors.push('items_config.items.consumables 必须是数组')
					} else {
						categories.consumables.forEach((consumable: any, index: number) => {
							if (!consumable || typeof consumable !== 'object') {
								errors.push(`消耗品[${index}]配置必须是对象`)
								return
							}
							if (!consumable.name || typeof consumable.name !== 'string' || consumable.name.trim().length === 0) {
								errors.push(`消耗品[${index}]缺少名称`)
							}
							if (!consumable.properties || typeof consumable.properties !== 'object') {
								errors.push(`消耗品[${index}]缺少属性配置`)
							} else {
								const properties = consumable.properties
								if (!properties.effect_type || typeof properties.effect_type !== 'string' || properties.effect_type.trim().length === 0) {
									errors.push(`消耗品[${index}]缺少效果类型`)
								}
								if (typeof properties.effect_value !== 'number' || !Number.isFinite(properties.effect_value)) {
									errors.push(`消耗品[${index}]效果值必须是数字`)
								}
								if (properties.cure_bleed !== undefined && (typeof properties.cure_bleed !== 'number' || !Number.isFinite(properties.cure_bleed))) {
									errors.push(`消耗品[${index}]治愈流血字段必须是数字`)
								}
							}
						})
					}

					if (!Array.isArray(categories.upgraders)) {
						errors.push('items_config.items.upgraders 必须是数组')
					} else {
						categories.upgraders.forEach((upgrader: any, index: number) => {
							if (!upgrader || typeof upgrader !== 'object') {
								errors.push(`升级器[${index}]配置必须是对象`)
								return
							}
							if (!upgrader.internal_name || typeof upgrader.internal_name !== 'string' || upgrader.internal_name.trim().length === 0) {
								errors.push(`升级器[${index}]缺少内部名称`)
							}
							if (!Array.isArray(upgrader.display_names) || upgrader.display_names.length === 0) {
								errors.push(`升级器[${index}]显示名称必须是非空数组`)
							}
						})
					}
				}

				if (Object.prototype.hasOwnProperty.call(itemsConfig, 'upgrade_recipes')) {
					if (!itemsConfig.upgrade_recipes || typeof itemsConfig.upgrade_recipes !== 'object') {
						errors.push('items_config.upgrade_recipes 必须是对象')
					}
				}
			}
		}

		if (config.display_names !== undefined) {
			if (!config.display_names || typeof config.display_names !== 'object') {
				errors.push('display_names 必须是对象')
			} else {
				const displayFields: Array<[string, string]> = [
					['player_max_life', '玩家最大生命值显示名称'],
					['player_max_strength', '玩家最大体力值显示名称'],
					['player_daily_life_recovery', '玩家每日生命恢复显示名称'],
					['player_daily_strength_recovery', '玩家每日体力恢复显示名称'],
					['player_search_cooldown', '玩家搜索冷却显示名称'],
					['action_move', '行动-移动显示名称'],
					['action_search', '行动-搜索显示名称'],
					['action_pick', '行动-拾取显示名称'],
					['action_attack', '行动-攻击显示名称'],
					['action_equip', '行动-装备显示名称'],
					['action_use', '行动-使用显示名称'],
					['action_throw', '行动-丢弃显示名称'],
					['action_deliver', '行动-传音显示名称'],
					['player_unarmed_damage', '玩家挥拳伤害显示名称'],
					['rest_life_recovery', '静养生命恢复显示名称'],
					['rest_max_moves', '静养最大移动次数显示名称']
				]

				for (const [field, label] of displayFields) {
					if (!Object.prototype.hasOwnProperty.call(config.display_names, field)) {
						errors.push(`缺少 display_names.${field} 字段`)
						continue
					}
					const value = config.display_names[field]
					if (typeof value !== 'string' || value.trim().length === 0) {
						errors.push(`${label}必须是非空字符串`)
					}
				}
			}
		}

		return {
			isValid: errors.length === 0,
			errors,
			missingSections
		}
	}

	parseTeammateBehavior(value: number) {
		return {
			noHarm: (value & 1) !== 0,
			noSearch: (value & 2) !== 0,
			canViewStatus: (value & 4) !== 0,
			canTransferItems: (value & 8) !== 0
		}
	}

	private findDuplicates<T>(values: T[]): T[] {
		const seen = new Set<T>()
		const duplicates = new Set<T>()
		values.forEach(value => {
			if (seen.has(value)) {
				duplicates.add(value)
			} else {
				seen.add(value)
			}
		})
		return Array.from(duplicates)
	}

	private findMissingSafePlaces(safePlaces: string[], places: string[]): string[] {
		const placeSet = new Set(places)
		return safePlaces.filter(place => !placeSet.has(place))
	}
}
