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

// 玩家接口
export interface Player {
  id: string;
  name: string;
  location: string;
  life: number;
  strength: number;
  inventory: Item[];
  equipped_item: string | null;
  hand_item: string | null;
  last_search_result: SearchResult | null;
  is_alive: boolean;
  is_bound: boolean;
  rest_mode: boolean;
  last_search_time: string | null;
  votes: number;
}

// 地点接口
export interface Place {
  name: string;
  players: string[]; // 玩家ID列表
  items: Item[];
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
}

// 游戏数据接口
export interface GameData {
  players: Record<string, Player>;
  places: Record<string, Place>;
}

// 消息类型枚举
export type MessageType = 'system_notice' | 'user_directed';

// 动作结果接口
export interface ActionResult {
  data: Record<string, any>;
  log_message: string;
  message_type: MessageType;
  timestamp: string;
}

// 导演视角的游戏状态接口
export interface DirectorGameState {
  global_state: GlobalState;
  game_data: GameData;
  action_result: ActionResult | null;
}

// 空投物品接口
export interface AirdropItem {
  id: string;
  name: string;
}