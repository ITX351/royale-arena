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
}

// 导演视角的游戏数据接口
export interface DirectorGameData {
  players: Record<string, Player>;
  places: Record<string, DirectorPlace>;
}

// 玩家视角的游戏数据接口
export interface ActorGameData {
  player: Player;
  places: Record<string, ActorPlace>;
}

// 消息类型枚举
export type MessageType = 'SystemNotice' | 'UserDirected' | 'Info';

// 动作结果接口
export interface ActionResult {
  data: Record<string, any>;
  log_message: string;
  message_type: MessageType;
  timestamp: string;
  broadcast_to_director?: boolean; // 可选字段，后端传输时可能不包含
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