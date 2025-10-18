// 导演视角的游戏状态类型定义

// 游戏阶段枚举
export type GamePhase = 'day' | 'night';

// 物品类型枚举
export type ItemType = 'weapon' | 'consumable' | 'equipment';

// 物品接口
export interface Item {
  id: string;
  name: string;
  item_type: ItemType;
  properties: Record<string, any>;
}

// 导演视角的玩家接口
export interface Player {
  id: string;
  name: string;
  location: string;
  life: number;
  strength: number;
  max_life: number;
  max_strength: number;
  inventory: Item[];
  equipped_weapon: Item | null; // 修改：单槽位武器
  equipped_armor: Item | null; // 修改：单槽位防具
  last_search_result: SearchResult | null;
  is_alive: boolean;
  is_bound: boolean;
  rest_mode: boolean;
  rest_life_recovery: number;
  rest_moves_used: number;
  last_search_time: string | null;
  votes: number;
  team_id: number | null;
  bleed_damage: number;
  bleed_rounds_remaining: number;
}

// 玩家视角的玩家列表
export interface ActorPlayer {
  id: string;
  name: string;
}

// 导演视角的地点接口
export interface DirectorPlace {
  name: string;
  players: string[]; // 玩家ID列表
  items: Item[];
  is_destroyed: boolean;
}

// 玩家视角的地点接口（不包含其他玩家信息和物品信息）
export interface ActorPlace {
  name: string;
  is_destroyed: boolean;
}

// 搜索结果类型枚举
export type SearchResultType = 'player' | 'item';

// 搜索结果接口
export interface SearchResult {
  target_type: SearchResultType;
  target_id: string;
  target_name: string;
  is_visible: boolean;
}

// 全局游戏状态接口
export interface GlobalState {
  game_phase: GamePhase;
  weather: number;
  night_start_time: string | null;
  night_end_time: string | null;
  next_night_destroyed_places: string[];
  rules_config: Record<string, any>; // 后端传递的规则配置
}

// 导演视角的游戏数据接口
export interface DirectorGameData {
  players: Record<string, Player>;
  places: Record<string, DirectorPlace>;
}

// 玩家视角的游戏数据接口
export interface ActorGameData {
  player: Player;
  actor_players: Record<string, ActorPlayer>;
  actor_places: Record<string, ActorPlace>;
}

// 消息类型枚举
export type MessageType = 'SystemNotice' | 'UserDirected' | 'Info';

// 动作结果接口
export interface ActionResult {
  data?: Record<string, any>;
  log_message: string;
  message_type: MessageType;
  timestamp: string;
}

// 导演视角的游戏状态接口
export interface DirectorGameState {
  global_state: GlobalState;
  game_data: DirectorGameData;
  action_result: ActionResult | null;
}

// 玩家视角的游戏状态接口
export interface ActorGameState {
  global_state: GlobalState;
  game_data: ActorGameData;
  action_result: ActionResult | null;
}

// 空投物品接口
export interface AirdropItem {
  id: string;
  name: string;
}

// 装备分类接口
export interface EquipmentSlots {
  weapons: Item[];
  armors: Item[];
  other: Item[];
}

// 玩家状态详情接口（用于导演界面展示）
export interface PlayerStatusDetail {
  basic: {
    id: string;
    name: string;
    location: string;
    team_id: number | null;
  };
  health: {
    life: number;
    max_life: number;
    strength: number;
    max_strength: number;
    is_alive: boolean;
  };
  status: {
    is_bound: boolean;
    rest_mode: boolean;
    rest_life_recovery: number;
    rest_moves_used: number;
  };
  effects: {
    bleed_damage: number;
    bleed_rounds_remaining: number;
  };
  equipment: {
    weapon: Item | null;
    armor: Item | null;
    inventory: Item[];
  };
  activity: {
    last_search_result: SearchResult | null;
    last_search_time: string | null;
    votes: number;
  };
}

// 玩家状态统计接口
export interface PlayerStats {
  totalPlayers: number;
  alivePlayers: number;
  deadPlayers: number;
  boundPlayers: number;
  restingPlayers: number;
  bleedingPlayers: number;
}

// 游戏规则配置接口
export interface GameRuleConfig {
  map: {
    places: string[];
    safe_places: string[];
  };
  player: {
    max_life: number;
    max_strength: number;
    daily_strength_recovery: number;
    search_cooldown: number;
    max_backpack_items: number;
    unarmed_damage: number;  // 挥拳伤害
  };
  action_costs: {
    move: number;
    search: number;
    pick: number;
    attack: number;
    equip: number;
    use: number;
    throw: number;
    deliver: number;
  };
  rest_mode: {
    life_recovery: number;
    max_moves: number;
  };
  teammate_behavior: number;
  death_item_disposition: string;
}