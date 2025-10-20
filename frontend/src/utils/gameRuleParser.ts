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
    unarmedDamage: number;  // 挥拳伤害
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
  items: {
    rarityLevels: Array<{
      internalName: string;
      displayName: string;
      prefix: string;
      isAirdropped: boolean;
    }>;
    weapons: Array<{
      internalName: string;
      displayNames: string[];
      rarity: string;
      category: string;
      properties: {
        damage: number;
        uses?: number;
        votes: number;
        aoeDamage?: number;
        bleedDamage?: number;
      };
    }>;
    otherItems: Array<{
      name: string;
      category: string;
      properties: {
        uses?: number;
        votes: number;
        targets?: number;
        damage?: number;
        usesNight?: number;
      };
    }>;
    upgraders: Array<{
      internalName: string;
      displayNames: string[];
      rarity: string;
    }>;
    upgradeRecipes: Record<string, Array<{
      result: string;
      ingredients: string[];
    }>>;
    consumables: Array<{
      name: string;
      effectType: string;
      effectValue: number;
      cureBleed: number;
    }>;
  };
  
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

// 定义武器属性接口
interface WeaponProperties {
  damage: number;
  uses?: number;
  votes: number;
  aoeDamage?: number;
  bleedDamage?: number;
}

// 定义护甲属性接口
interface ArmorProperties {
  defense: number;
  uses?: number;
  votes: number;
}

// 定义其他物品属性接口
interface OtherItemProperties {
  uses?: number;
  votes: number;
  targets?: number;
  damage?: number;
  usesNight?: number;
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
      items: {
        rarityLevels: [],
        weapons: [],
        otherItems: [],
        upgraders: [],
        upgradeRecipes: {},
        consumables: []
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
    if (rulesConfig.items) {
      // 解析稀有度级别
      if (rulesConfig.items.rarity_levels) {
        parsedRules.items.rarityLevels = rulesConfig.items.rarity_levels.map((level: any) => ({
          internalName: level.internal_name || '',
          displayName: level.display_name || '',
          prefix: level.prefix || '',
          isAirdropped: level.is_airdropped !== undefined ? level.is_airdropped : true
        }));
      }

      // 解析武器
      if (rulesConfig.items.weapons) {
        parsedRules.items.weapons = rulesConfig.items.weapons.map((weapon: any) => {
          // 解析武器属性
          const properties: WeaponProperties = {
            damage: weapon.properties?.damage || 0,
            votes: weapon.properties?.votes || 0
          };
          
          // 如果有使用次数，则添加到属性中
          if (weapon.properties?.uses !== undefined) {
            properties.uses = weapon.properties.uses;
          }
          
          // 如果是橙色武器，解析特殊属性
          if (weapon.rarity === 'legendary' && weapon.properties) {
            if (weapon.properties.aoe_damage !== undefined) {
              properties.aoeDamage = weapon.properties.aoe_damage;
            }
            if (weapon.properties.bleed_damage !== undefined) {
              properties.bleedDamage = weapon.properties.bleed_damage;
            }
          }
          
          return {
            internalName: weapon.internal_name || '',
            displayNames: weapon.display_names || [],
            rarity: weapon.rarity || '',
            category: 'weapon',
            properties
          };
        });
      }

      // 解析护甲
      if (rulesConfig.items.armors) {
        const armors = rulesConfig.items.armors.map((armor: any) => {
          // 解析护甲属性
          const properties: ArmorProperties = {
            defense: armor.properties?.defense || 0,
            votes: armor.properties?.votes || 0
          };
          
          // 如果有使用次数，则添加到属性中
          if (armor.properties?.uses !== undefined) {
            properties.uses = armor.properties.uses;
          }
          
          return {
            internalName: armor.internal_name || '',
            displayNames: armor.display_names || [],
            rarity: armor.rarity || '',
            category: 'equipment_armor',
            properties
          };
        });
        parsedRules.items.weapons = [...parsedRules.items.weapons, ...armors];
      }

      // 解析其他物品
      if (rulesConfig.items.other_items) {
        parsedRules.items.otherItems = rulesConfig.items.other_items.map((item: any) => {
          // 解析其他物品属性
          const properties: OtherItemProperties = {
            votes: item.properties?.votes || 0
          };
          
          // 如果有使用次数，则添加到属性中
          if (item.properties?.uses !== undefined) {
            properties.uses = item.properties.uses;
          }
          
          // 如果有夜间使用次数限制，则添加到属性中
          if (item.properties?.uses_night !== undefined) {
            properties.usesNight = item.properties.uses_night;
          }

          // 如果有目标数量，则添加到属性中
          if (item.properties?.targets !== undefined) {
            properties.targets = item.properties.targets;
          }
          
          // 如果有伤害值，则添加到属性中
          if (item.properties?.damage !== undefined) {
            properties.damage = item.properties.damage;
          }
          
          return {
            name: item.name || '',
            category: item.category || '',
            properties
          };
        });
      }

      // 解析升级道具
      if (rulesConfig.items.upgraders) {
        parsedRules.items.upgraders = rulesConfig.items.upgraders.map((upgrader: any) => ({
          internalName: upgrader.internal_name || '',
          displayNames: upgrader.display_names || [],
          rarity: upgrader.rarity || ''
        }));
      }

      // 解析合成配方
      if (rulesConfig.items.upgrade_recipes) {
        parsedRules.items.upgradeRecipes = rulesConfig.items.upgrade_recipes;
      }

      // 解析消耗品
      if (rulesConfig.items.consumables) {
        parsedRules.items.consumables = rulesConfig.items.consumables.map((consumable: any) => ({
          name: consumable.name || '',
          effectType: consumable.effect_type || '',
          effectValue: consumable.effect_value || 0,
          cureBleed: consumable.cure_bleed || 0
        }));
      }
    } else {
      parsedRules.missingSections.push('items');
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
    const requiredFields = ['map', 'player', 'action_costs', 'rest_mode', 'teammate_behavior', 'items'];
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
    if (rulesConfig.items) {
      if (!Array.isArray(rulesConfig.items.rarity_levels)) {
        errors.push('稀有度级别必须是数组');
      }
      
      // 检查武器配置
      if (rulesConfig.items.weapons) {
        if (!Array.isArray(rulesConfig.items.weapons)) {
          errors.push('武器配置必须是数组');
        } else {
          rulesConfig.items.weapons.forEach((weapon: any, index: number) => {
            if (!weapon.internal_name) {
              errors.push(`武器[${index}]缺少内部名称`);
            }
            if (!weapon.display_names || !Array.isArray(weapon.display_names)) {
              errors.push(`武器[${index}]显示名称必须是数组`);
            }
            if (!weapon.rarity) {
              errors.push(`武器[${index}]缺少稀有度`);
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
            }
          });
        }
      }
      
      // 检查护甲配置
      if (rulesConfig.items.armors) {
        if (!Array.isArray(rulesConfig.items.armors)) {
          errors.push('护甲配置必须是数组');
        } else {
          rulesConfig.items.armors.forEach((armor: any, index: number) => {
            if (!armor.internal_name) {
              errors.push(`护甲[${index}]缺少内部名称`);
            }
            if (!armor.display_names || !Array.isArray(armor.display_names)) {
              errors.push(`护甲[${index}]显示名称必须是数组`);
            }
            if (!armor.rarity) {
              errors.push(`护甲[${index}]缺少稀有度`);
            }
            if (!armor.properties) {
              errors.push(`护甲[${index}]缺少属性配置`);
            } else {
              if (typeof armor.properties.defense !== 'number') {
                errors.push(`护甲[${index}]防御值必须是数字`);
              }
              if (typeof armor.properties.votes !== 'number') {
                errors.push(`护甲[${index}]票数必须是数字`);
              }
              if (armor.properties.uses !== undefined && typeof armor.properties.uses !== 'number') {
                errors.push(`护甲[${index}]使用次数必须是数字`);
              }
            }
          });
        }
      }
      
      // 检查其他物品配置
      if (rulesConfig.items.other_items) {
        if (!Array.isArray(rulesConfig.items.other_items)) {
          errors.push('其他物品配置必须是数组');
        } else {
          rulesConfig.items.other_items.forEach((item: any, index: number) => {
            if (!item.name) {
              errors.push(`其他物品[${index}]缺少名称`);
            }
            if (!item.category) {
              errors.push(`其他物品[${index}]缺少分类`);
            }
            if (!item.properties) {
              errors.push(`其他物品[${index}]缺少属性配置`);
            } else {
              if (typeof item.properties.votes !== 'number') {
                errors.push(`其他物品[${index}]票数必须是数字`);
              }
              if (item.properties.uses !== undefined && typeof item.properties.uses !== 'number') {
                errors.push(`其他物品[${index}]使用次数必须是数字`);
              }
              if (item.properties.targets !== undefined && typeof item.properties.targets !== 'number') {
                errors.push(`其他物品[${index}]目标数量必须是数字`);
              }
              if (item.properties.damage !== undefined && typeof item.properties.damage !== 'number') {
                errors.push(`其他物品[${index}]伤害值必须是数字`);
              }
              if (item.properties.uses_night !== undefined && typeof item.properties.uses_night !== 'number') {
                errors.push(`其他物品[${index}]每晚使用次数必须是数字`);
              }
            }
          });
        }
      }
      
      // 检查消耗品配置
      if (rulesConfig.items.consumables) {
        if (!Array.isArray(rulesConfig.items.consumables)) {
          errors.push('消耗品配置必须是数组');
        } else {
          rulesConfig.items.consumables.forEach((consumable: any, index: number) => {
            if (!consumable.name) {
              errors.push(`消耗品[${index}]缺少名称`);
            }
            if (!consumable.effect_type) {
              errors.push(`消耗品[${index}]缺少效果类型`);
            }
            if (typeof consumable.effect_value !== 'number') {
              errors.push(`消耗品[${index}]效果值必须是数字`);
            }
            if (consumable.cure_bleed !== undefined && typeof consumable.cure_bleed !== 'boolean') {
              errors.push(`消耗品[${index}]治愈流血状态必须是布尔值`);
            }
          });
        }
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
}