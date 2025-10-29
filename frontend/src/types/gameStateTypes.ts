// 导演视角的游戏状态类型定义

// 物品类型分类
export type ItemCategory = 'weapon' | 'armor' | 'consumable' | 'utility' | 'upgrader';

// 物品类型载荷
export interface ItemType {
  type: ItemCategory;
  properties?: Record<string, any>;
}

// 物品接口
export interface Item {
  id: string;
  name: string;
  internal_name: string | null;
  rarity: string | null;
  item_type: ItemType;
}

// 导演视角的玩家接口
export interface Player {
  id: string;
  name: string;
  password: string;
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
  rest_moves_used: number;
  last_search_time: string | null;
  team_id: number | null;
  bleed_damage: number;
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
  weather: number;
  night_start_time: string | null;
  night_end_time: string | null;
  next_night_destroyed_places: string[];
  rules_config: Record<string, any>; // 后端传递的规则配置
  server_now?: string; // 后端服务器当前时间戳（ISO字符串）
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
  id?: string; // 后端API返回的ID或前端为WebSocket消息添加的UUID
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
