import { findDuplicateItemNames, parseItemsConfig } from './itemConfigUtils';
import type { NormalizedItemsConfig } from './itemConfigUtils';

// 定义规则解析结果接口
export interface ParsedGameRules {
  // 基础规则
  map: {
    places: string[];
    safePlaces: string[];
  };
  player: {
    maxLife: number;
    maxStrength: number;
    dailyLifeRecovery: number;
    dailyStrengthRecovery: number;
    searchCooldown: number;
    maxBackpackItems: number;
    unarmedDamage: number; // 挥拳伤害
  };
  actionCosts: {
    move: number;
    search: number;
    pick: number;
    attack: number;
    equip: number;
    use: number;
    throw: number;
    deliver: number;
  };
  restMode: {
    lifeRecovery: number;
    strengthRecovery: number;
    maxMoves: number;
  };
  deathItemDisposition: string; // 死亡后物品去向
  teammateBehavior: number;
  parsedTeammateBehaviors: {
    noHarm: boolean;
    noSearch: boolean;
    canViewStatus: boolean;
    canTransferItems: boolean;
  }; // 解析后的队友行为效果

  // 物品规则
  itemsConfig: NormalizedItemsConfig;

  // 显示名称配置
  displayNames: {
    playerMaxLife: string;
    playerMaxStrength: string;
    playerDailyLifeRecovery: string;
    playerDailyStrengthRecovery: string;
    playerSearchCooldown: string;
    actionMove: string;
    actionSearch: string;
    actionPick: string;
    actionAttack: string;
    actionEquip: string;
    actionUse: string;
    actionThrow: string;
    actionDeliver: string;
    playerUnarmedDamage: string;
    restLifeRecovery: string;
    restMaxMoves: string;
  };

  // 解析状态
  parsingIssues: string[];
  missingSections: string[];
}

// 定义规则解析器类
export class GameRuleParser {
  parse(rulesConfig: any): ParsedGameRules {
    const parsedRules: ParsedGameRules = {
      map: {
        places: [],
        safePlaces: []
      },
      player: {
        maxLife: 100,
        maxStrength: 100,
        dailyLifeRecovery: 0,
        dailyStrengthRecovery: 40,
        searchCooldown: 30,
        maxBackpackItems: 4,
        unarmedDamage: 5
      },
      actionCosts: {
        move: 5,
        search: 5,
        pick: 0,
        attack: 0,
        equip: 0,
        use: 0,
        throw: 0,
        deliver: 105
      },
      restMode: {
        lifeRecovery: 25,
        strengthRecovery: 1000,
        maxMoves: 1
      },
      deathItemDisposition: "killer_takes_loot", // 默认值
      teammateBehavior: 0,
      parsedTeammateBehaviors: {
        noHarm: false,
        noSearch: false,
        canViewStatus: false,
        canTransferItems: false
      },
      itemsConfig: {
        rarityLevels: [],
        items: {
          weapons: [],
          armors: [],
          utilities: [],
          consumables: [],
          upgraders: []
        },
        upgradeRecipes: {}
      },
      displayNames: {
        playerMaxLife: "生命值",
        playerMaxStrength: "体力值",
        playerDailyLifeRecovery: "每日生命恢复",
        playerDailyStrengthRecovery: "每日体力恢复",
        playerSearchCooldown: "搜索冷却时间",
        actionMove: "移动",
        actionSearch: "搜索",
        actionPick: "拾取",
        actionAttack: "攻击",
        actionEquip: "装备",
        actionUse: "使用",
        actionThrow: "丢弃",
        actionDeliver: "传音",
        playerUnarmedDamage: "挥拳伤害",
        restLifeRecovery: "生命恢复",
        restMaxMoves: "最大移动次数"
      },
      parsingIssues: [],
      missingSections: []
    };

    // 解析地图配置
    if (rulesConfig.map) {
      parsedRules.map.places = rulesConfig.map.places || [];
      parsedRules.map.safePlaces = rulesConfig.map.safe_places || [];
    } else {
      parsedRules.missingSections.push('map');
    }

    // 解析玩家配置
    if (rulesConfig.player) {
      parsedRules.player.maxLife = rulesConfig.player.max_life || 100;
      parsedRules.player.maxStrength = rulesConfig.player.max_strength || 100;
      parsedRules.player.dailyLifeRecovery = rulesConfig.player.daily_life_recovery || 0;
      parsedRules.player.dailyStrengthRecovery = rulesConfig.player.daily_strength_recovery || 40;
      parsedRules.player.searchCooldown = rulesConfig.player.search_cooldown || 30;
      parsedRules.player.maxBackpackItems = rulesConfig.player.max_backpack_items || 4;
      parsedRules.player.unarmedDamage = rulesConfig.player.unarmed_damage || 5;  // 从 player 配置读取
    } else {
      parsedRules.missingSections.push('player');
    }

    // 解析行动消耗配置
    if (rulesConfig.action_costs) {
      parsedRules.actionCosts.move = rulesConfig.action_costs.move || 5;
      parsedRules.actionCosts.search = rulesConfig.action_costs.search || 5;
      parsedRules.actionCosts.pick = rulesConfig.action_costs.pick || 0;
      parsedRules.actionCosts.attack = rulesConfig.action_costs.attack || 0;
      parsedRules.actionCosts.equip = rulesConfig.action_costs.equip || 0;
      parsedRules.actionCosts.use = rulesConfig.action_costs.use || 0;
      parsedRules.actionCosts.throw = rulesConfig.action_costs.throw || 0;
      parsedRules.actionCosts.deliver = rulesConfig.action_costs.deliver || 105;
    } else {
      parsedRules.missingSections.push('action_costs');
    }

    // 解析静养模式配置
    if (rulesConfig.rest_mode) {
      parsedRules.restMode.lifeRecovery = rulesConfig.rest_mode.life_recovery || 25;
      parsedRules.restMode.strengthRecovery = rulesConfig.rest_mode.strength_recovery ?? 1000;
      parsedRules.restMode.maxMoves = rulesConfig.rest_mode.max_moves || 1;
    } else {
      parsedRules.missingSections.push('rest_mode');
    }

    // 解析死亡后物品去向规则
    if (typeof rulesConfig.death_item_disposition === 'string') {
      // 验证值是否为允许的选项之一
      const allowedDispositions = ["killer_takes_loot", "drop_to_ground", "vanish_completely"];
      if (allowedDispositions.includes(rulesConfig.death_item_disposition)) {
        parsedRules.deathItemDisposition = rulesConfig.death_item_disposition;
      } else {
        parsedRules.parsingIssues.push(`无效的死亡物品处理方式: ${rulesConfig.death_item_disposition}，使用默认值`);
        parsedRules.deathItemDisposition = "killer_takes_loot"; // 默认值
      }
    } else {
      parsedRules.deathItemDisposition = "killer_takes_loot"; // 默认值
    }

    // 解析队友行为规则
    if (typeof rulesConfig.teammate_behavior === 'number') {
      parsedRules.teammateBehavior = rulesConfig.teammate_behavior;
      // 解析队友行为的位运算值
      parsedRules.parsedTeammateBehaviors = this.parseTeammateBehavior(rulesConfig.teammate_behavior);
    } else {
      parsedRules.missingSections.push('teammate_behavior');
    }

    // 解析物品系统配置
    if (rulesConfig.items_config) {
      const { config: parsedItemsConfig, issues: itemIssues } = parseItemsConfig(rulesConfig.items_config);
      parsedRules.itemsConfig = parsedItemsConfig;
      parsedRules.parsingIssues.push(...itemIssues);

      const duplicateNames = findDuplicateItemNames(parsedItemsConfig);
      if (duplicateNames.length > 0) {
        parsedRules.parsingIssues.push(`规则JSON中发现重复的物品名称: ${duplicateNames.join(', ')}`);
      }
    } else {
      parsedRules.missingSections.push('items_config');
    }

    // 解析显示名称配置
    if (rulesConfig.display_names) {
      parsedRules.displayNames.playerMaxLife = rulesConfig.display_names.player_max_life || "生命值";
      parsedRules.displayNames.playerMaxStrength = rulesConfig.display_names.player_max_strength || "体力值";
      parsedRules.displayNames.playerDailyLifeRecovery = rulesConfig.display_names.player_daily_life_recovery || "每日生命恢复";
      parsedRules.displayNames.playerDailyStrengthRecovery = rulesConfig.display_names.player_daily_strength_recovery || "每日体力恢复";
      parsedRules.displayNames.playerSearchCooldown = rulesConfig.display_names.player_search_cooldown || "搜索冷却时间";
      parsedRules.displayNames.actionMove = rulesConfig.display_names.action_move || "移动";
      parsedRules.displayNames.actionSearch = rulesConfig.display_names.action_search || "搜索";
      parsedRules.displayNames.actionPick = rulesConfig.display_names.action_pick || "拾取";
      parsedRules.displayNames.actionAttack = rulesConfig.display_names.action_attack || "攻击";
      parsedRules.displayNames.actionEquip = rulesConfig.display_names.action_equip || "装备";
      parsedRules.displayNames.actionUse = rulesConfig.display_names.action_use || "使用";
      parsedRules.displayNames.actionThrow = rulesConfig.display_names.action_throw || "丢弃";
      parsedRules.displayNames.actionDeliver = rulesConfig.display_names.action_deliver || "传音";
      parsedRules.displayNames.playerUnarmedDamage = rulesConfig.display_names.player_unarmed_damage || "挥拳伤害";
      parsedRules.displayNames.restLifeRecovery = rulesConfig.display_names.rest_life_recovery || "生命恢复";
      parsedRules.displayNames.restMaxMoves = rulesConfig.display_names.rest_max_moves || "最大移动次数";
    }

    return parsedRules;
  }
  
  validate(rulesConfig: any): { isValid: boolean; errors: string[] } {
    const errors: string[] = [];
    
    // 检查必需的顶层字段
  const requiredFields = ['map', 'player', 'action_costs', 'rest_mode', 'teammate_behavior', 'items_config'];
    for (const field of requiredFields) {
      if (!rulesConfig[field]) {
        errors.push(`缺少必需字段: ${field}`);
      }
    }
    
    // 检查死亡后物品去向字段
    if (rulesConfig.death_item_disposition !== undefined) {
      if (typeof rulesConfig.death_item_disposition !== 'string') {
        errors.push('死亡后物品去向必须是字符串');
      } else {
        const allowedDispositions = ["killer_takes_loot", "drop_to_ground", "vanish_completely"];
        if (!allowedDispositions.includes(rulesConfig.death_item_disposition)) {
          errors.push('死亡后物品去向值无效，必须是: killer_takes_loot, drop_to_ground, vanish_completely 之一');
        }
      }
    }
    
    // 检查地图配置
    if (rulesConfig.map) {
      if (!Array.isArray(rulesConfig.map.places)) {
        errors.push('地图地点必须是数组');
      }
      if (!Array.isArray(rulesConfig.map.safe_places)) {
        errors.push('安全地点必须是数组');
      }
      if (Array.isArray(rulesConfig.map.places)) {
        const placeDuplicates = this.findDuplicates(rulesConfig.map.places as string[]);
        if (placeDuplicates.length > 0) {
          errors.push(`地图地点列表存在重复项: ${placeDuplicates.map(String).join(', ')}`);
        }
      }
      if (Array.isArray(rulesConfig.map.safe_places)) {
        const safePlaceDuplicates = this.findDuplicates(rulesConfig.map.safe_places as string[]);
        if (safePlaceDuplicates.length > 0) {
          errors.push(`安全地点列表存在重复项: ${safePlaceDuplicates.map(String).join(', ')}`);
        }
      }
      if (Array.isArray(rulesConfig.map.places) && Array.isArray(rulesConfig.map.safe_places)) {
        const missingSafePlaces = this.findMissingSafePlaces(
          rulesConfig.map.safe_places as string[],
          rulesConfig.map.places as string[]
        );
        if (missingSafePlaces.length > 0) {
          errors.push(`安全地点列表包含未在地点名单中的地点: ${missingSafePlaces.map(String).join(', ')}`);
        }
      }
    }
    
    // 检查玩家配置
    if (rulesConfig.player) {
      if (typeof rulesConfig.player.max_life !== 'number') {
        errors.push('玩家最大生命值必须是数字');
      }
      if (typeof rulesConfig.player.max_strength !== 'number') {
        errors.push('玩家最大体力值必须是数字');
      }
      if (rulesConfig.player.max_backpack_items !== undefined && 
          typeof rulesConfig.player.max_backpack_items !== 'number') {
        errors.push('玩家背包最大物品数必须是数字');
      }
    }
    
    // 检查物品系统
    if (rulesConfig.items_config) {
      const itemsConfig = rulesConfig.items_config;

      if (!Array.isArray(itemsConfig.rarity_levels)) {
        errors.push('items_config.rarity_levels 必须是数组');
      }

      if (!itemsConfig.items || typeof itemsConfig.items !== 'object') {
        errors.push('items_config.items 必须存在且为对象');
      } else {
        const categories = itemsConfig.items;

        if (categories.weapons !== undefined) {
          if (!Array.isArray(categories.weapons)) {
            errors.push('items_config.items.weapons 必须是数组');
          } else {
            categories.weapons.forEach((weapon: any, index: number) => {
              if (!weapon.internal_name) {
                errors.push(`武器[${index}]缺少内部名称`);
              }
              if (!weapon.display_names || !Array.isArray(weapon.display_names)) {
                errors.push(`武器[${index}]显示名称必须是数组`);
              }
              if (!weapon.properties) {
                errors.push(`武器[${index}]缺少属性配置`);
              } else {
                if (typeof weapon.properties.damage !== 'number') {
                  errors.push(`武器[${index}]伤害值必须是数字`);
                }
                if (typeof weapon.properties.votes !== 'number') {
                  errors.push(`武器[${index}]票数必须是数字`);
                }
                if (weapon.properties.uses !== undefined && typeof weapon.properties.uses !== 'number') {
                  errors.push(`武器[${index}]使用次数必须是数字`);
                }
                if (weapon.properties.aoe_damage !== undefined && typeof weapon.properties.aoe_damage !== 'number') {
                  errors.push(`武器[${index}]范围伤害必须是数字`);
                }
                if (weapon.properties.bleed_damage !== undefined && typeof weapon.properties.bleed_damage !== 'number') {
                  errors.push(`武器[${index}]流血伤害必须是数字`);
                }
              }
            });
          }
        }

        if (categories.armors !== undefined) {
          if (!Array.isArray(categories.armors)) {
            errors.push('items_config.items.armors 必须是数组');
          } else {
            categories.armors.forEach((armor: any, index: number) => {
              if (!armor.internal_name) {
                errors.push(`防具[${index}]缺少内部名称`);
              }
              if (!armor.display_names || !Array.isArray(armor.display_names)) {
                errors.push(`防具[${index}]显示名称必须是数组`);
              }
              if (!armor.properties) {
                errors.push(`防具[${index}]缺少属性配置`);
              } else {
                if (typeof armor.properties.defense !== 'number') {
                  errors.push(`防具[${index}]防御值必须是数字`);
                }
                if (typeof armor.properties.votes !== 'number') {
                  errors.push(`防具[${index}]票数必须是数字`);
                }
                if (armor.properties.uses !== undefined && typeof armor.properties.uses !== 'number') {
                  errors.push(`防具[${index}]使用次数必须是数字`);
                }
              }
            });
          }
        }

        if (categories.utilities !== undefined) {
          if (!Array.isArray(categories.utilities)) {
            errors.push('items_config.items.utilities 必须是数组');
          } else {
            categories.utilities.forEach((item: any, index: number) => {
              if (!item.name) {
                errors.push(`功能物品[${index}]缺少名称`);
              }
              if (!item.properties) {
                errors.push(`功能物品[${index}]缺少属性配置`);
              } else {
                if (!item.properties.category) {
                  errors.push(`功能物品[${index}]缺少分类`);
                }
                if (item.properties.votes !== undefined && typeof item.properties.votes !== 'number') {
                  errors.push(`功能物品[${index}]票数必须是数字`);
                }
                if (item.properties.uses !== undefined && typeof item.properties.uses !== 'number') {
                  errors.push(`功能物品[${index}]使用次数必须是数字`);
                }
                if (item.properties.targets !== undefined && typeof item.properties.targets !== 'number') {
                  errors.push(`功能物品[${index}]目标数量必须是数字`);
                }
                if (item.properties.damage !== undefined && typeof item.properties.damage !== 'number') {
                  errors.push(`功能物品[${index}]伤害值必须是数字`);
                }
                if (item.properties.uses_night !== undefined && typeof item.properties.uses_night !== 'number') {
                  errors.push(`功能物品[${index}]夜间使用次数必须是数字`);
                }
              }
            });
          }
        }

        if (categories.consumables !== undefined) {
          if (!Array.isArray(categories.consumables)) {
            errors.push('items_config.items.consumables 必须是数组');
          } else {
            categories.consumables.forEach((consumable: any, index: number) => {
              if (!consumable.name) {
                errors.push(`消耗品[${index}]缺少名称`);
              }
              if (!consumable.properties) {
                errors.push(`消耗品[${index}]缺少属性配置`);
              } else {
                if (!consumable.properties.effect_type) {
                  errors.push(`消耗品[${index}]缺少效果类型`);
                }
                if (typeof consumable.properties.effect_value !== 'number') {
                  errors.push(`消耗品[${index}]效果值必须是数字`);
                }
                if (consumable.properties.cure_bleed !== undefined && typeof consumable.properties.cure_bleed !== 'number') {
                  errors.push(`消耗品[${index}]治愈流血字段必须是数字`);
                }
              }
            });
          }
        }

        if (categories.upgraders !== undefined) {
          if (!Array.isArray(categories.upgraders)) {
            errors.push('items_config.items.upgraders 必须是数组');
          } else {
            categories.upgraders.forEach((upgrader: any, index: number) => {
              if (!upgrader.internal_name) {
                errors.push(`升级器[${index}]缺少内部名称`);
              }
              if (!upgrader.display_names || !Array.isArray(upgrader.display_names)) {
                errors.push(`升级器[${index}]显示名称必须是数组`);
              }
            });
          }
        }
      }

      if (itemsConfig.upgrade_recipes !== undefined && typeof itemsConfig.upgrade_recipes !== 'object') {
        errors.push('items_config.upgrade_recipes 必须是对象');
      }
    }
    
    return {
      isValid: errors.length === 0,
      errors
    };
  }
  
  /**
   * 解析队友行为的位运算值
   * @param value 位运算值
   * @returns 解析后的行为对象
   */
  parseTeammateBehavior(value: number) {
    return {
      noHarm: (value & 1) !== 0,           // 禁止队友伤害 (1)
      noSearch: (value & 2) !== 0,         // 禁止搜索到队友 (2)
      canViewStatus: (value & 4) !== 0,    // 允许查看队友状态 (4)
      canTransferItems: (value & 8) !== 0  // 允许赠送物品给队友 (8)
    };
  }

  private findDuplicates<T>(values: T[]): T[] {
    const seen = new Set<T>();
    const duplicates = new Set<T>();
    values.forEach((value) => {
      if (seen.has(value)) {
        duplicates.add(value);
      } else {
        seen.add(value);
      }
    });
    return Array.from(duplicates);
  }

  private findMissingSafePlaces(safePlaces: string[], places: string[]): string[] {
    const placeSet = new Set(places);
    return safePlaces.filter((place) => !placeSet.has(place));
  }
}