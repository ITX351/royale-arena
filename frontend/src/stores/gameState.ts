import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { ElNotification } from 'element-plus'
import type { 
  DirectorGameState, 
  ActorGameState,
  GlobalState, 
  DirectorGameData, 
  ActorGameData,
  Player, 
  DirectorPlace, 
  ActorPlace, 
  ActionResult 
} from '@/types/gameStateTypes'
import { webSocketService, type WebSocketEvent } from '@/services/webSocketService'

export const useGameStateStore = defineStore('gameState', () => {
  // 状态
  const gameState = ref<DirectorGameState | ActorGameState | null>(null)
  const connected = ref(false)
  const connecting = ref(false)
  const error = ref<string | null>(null)
  const logMessages = ref<ActionResult[]>([])
  const maxLogMessages = ref(100) // 最多保存100条日志消息

  // 计算属性
  const globalState = computed<GlobalState | null>(() => {
    return gameState.value?.global_state || null
  })

  const gameData = computed<DirectorGameData | ActorGameData | null>(() => {
    return gameState.value?.game_data || null
  })

  // 导演视角的计算属性
  const directorPlayers = computed<Record<string, Player>>(() => {
    if (!gameState.value || !('players' in gameState.value.game_data)) return {}
    return gameState.value.game_data.players || {}
  })

  const directorPlaces = computed<Record<string, DirectorPlace>>(() => {
    if (!gameState.value || !('players' in gameState.value.game_data)) return {}
    return gameState.value.game_data.places || {}
  })

  // 玩家视角的计算属性
  const actorPlayer = computed<Player | null>(() => {
    if (!gameState.value || !('player' in gameState.value.game_data)) return null
    return gameState.value.game_data.player || null
  })

  const actorPlaces = computed<Record<string, ActorPlace>>(() => {
    if (!gameState.value || !('player' in gameState.value.game_data)) return {}
    return gameState.value.game_data.places || {}
  })

  const actionResult = computed<ActionResult | null>(() => {
    return gameState.value?.action_result || null
  })

  const playerList = computed<Player[]>(() => {
    return Object.values(directorPlayers.value)
  })

  const directorPlaceList = computed<DirectorPlace[]>(() => {
    return Object.values(directorPlaces.value)
  })

  const actorPlaceList = computed<ActorPlace[]>(() => {
    return Object.values(actorPlaces.value)
  })

  // 操作
  const connect = async (gameId: string, password: string, userType: string) => {
    connecting.value = true
    error.value = null
    
    try {
      // 添加WebSocket事件监听器
      webSocketService.addEventListener(handleWebSocketEvent)
      
      // 连接到WebSocket，传递用户类型参数
      await webSocketService.connect(gameId, password, userType)
      connected.value = true
    } catch (err) {
      console.error('连接WebSocket失败:', err)
      error.value = err instanceof Error ? err.message : '连接失败'
      connected.value = false
    } finally {
      connecting.value = false
    }
  }

  const disconnect = () => {
    webSocketService.removeEventListener(handleWebSocketEvent)
    webSocketService.disconnect()
    connected.value = false
    connecting.value = false
  }

  const updateGameState = (newState: DirectorGameState | ActorGameState) => {
    gameState.value = newState
    
    // 如果有动作结果，只有非Info类型的消息才添加到日志消息中
    if (newState.action_result && newState.action_result.message_type !== 'Info') {
      addLogMessage(newState.action_result)
    }
  }

  const addLogMessage = (message: ActionResult) => {
    // 添加到日志消息列表开头
    logMessages.value.unshift(message)
    
    // 如果超过最大数量，移除最旧的消息
    if (logMessages.value.length > maxLogMessages.value) {
      logMessages.value = logMessages.value.slice(0, maxLogMessages.value)
    }
  }

  const clearLogMessages = () => {
    logMessages.value = []
  }

  const sendDirectorAction = (action: string, params: Record<string, any> = {}) => {
    webSocketService.sendDirectorAction(action, params)
  }

  const sendPlayerAction = (action: string, params: Record<string, any> = {}) => {
    const message = {
      type: 'player_action',
      data: {
        action,
        ...params
      }
    }
    webSocketService.sendMessage(message)
  }

  // 天气调节
  const updateWeather = (weather: number) => {
    // 验证天气值在有效范围内(0-2)
    if (weather < 0 || weather > 2) {
      console.error('天气值必须在0-2之间')
      return
    }
    // 根据后端要求使用正确的参数格式
    sendDirectorAction('weather', { weather: weather })
  }

  // 夜晚时间设置
  const setNightTime = (startTime: string | null, endTime: string | null) => {
    // 根据后端要求使用正确的参数格式
    if (startTime !== null) {
      sendDirectorAction('set_night_start_time', { timestamp: startTime })
    }
    if (endTime !== null) {
      sendDirectorAction('set_night_end_time', { timestamp: endTime })
    }
  }

  // 缩圈地点设置
  const setDestroyPlaces = (places: string[]) => {
    // 根据后端要求使用正确的参数格式
    sendDirectorAction('set_destroy_places', { places: places })
  }

  // 地点状态调整
  const togglePlaceStatus = (placeName: string, isDestroyed: boolean) => {
    // 根据后端要求使用正确的参数格式
    sendDirectorAction('modify_place', { 
      place_name: placeName, 
      is_destroyed: isDestroyed 
    })
  }

  // 设置玩家生命值（绝对值）
  const setPlayerLife = (playerId: string, life: number) => {
    sendDirectorAction('life', { player_id: playerId, life: life })
  }

  // 设置玩家体力值（绝对值）
  const setPlayerStrength = (playerId: string, strength: number) => {
    sendDirectorAction('strength', { player_id: playerId, strength: strength })
  }

  // 玩家移动
  const movePlayer = (playerId: string, targetPlace: string) => {
    sendDirectorAction('move_player', { player_id: playerId, target_place: targetPlace })
  }

  // 玩家捆绑/松绑
  const togglePlayerBinding = (playerId: string) => {
    // 先获取玩家当前状态来决定是捆绑还是松绑
    const player = directorPlayers.value[playerId]
    if (player) {
      if (player.is_bound) {
        // 松绑
        sendDirectorAction('unrope', { 
          player_id: playerId
        })
      } else {
        // 捆绑
        sendDirectorAction('rope', { 
          player_id: playerId
        })
      }
    }
  }

  const destroyPlace = (placeName: string) => {
    sendDirectorAction('destroy', { place: placeName })
  }

  const sendBroadcast = (message: string) => {
    sendDirectorAction('broadcast', { message })
  }

  const sendDirectorMessageToPlayer = (playerId: string, message: string) => {
    sendDirectorAction('message_to_player', { player_id: playerId, message })
  }

  // 批量空投（统一接口，支持单次和批量）
  const sendBatchAirdrop = (airdrops: Array<{ item_name: string, place_name: string }>) => {
    sendDirectorAction('batch_airdrop', { airdrops })
  }

  // 批量物品删除（统一接口，支持单个删除、地点清空和全场清空）
  const sendBatchItemDeletion = (deletions: Array<{ place_name: string, item_name?: string }>, clearAll: boolean = false) => {
    sendDirectorAction('batch_item_deletion', { deletions, clear_all: clearAll })
  }

  // 新增方法：向玩家添加物品（使用物品名称）
  const addPlayerItem = (playerId: string, itemName: string) => {
    sendDirectorAction('add_player_item', { player_id: playerId, item_name: itemName })
  }

  // 新增方法：从玩家移除物品（使用物品名称）
  const removePlayerItem = (playerId: string, itemName: string) => {
    sendDirectorAction('remove_player_item', { player_id: playerId, item_name: itemName })
  }

  const handleWebSocketEvent = (event: WebSocketEvent) => {
    switch (event.type) {
      case 'state_update':
        console.log('游戏状态更新:', event.data)
        updateGameState(event.data)
        break
      case 'action_result':
        console.log('追加日志:', event.data)
        //addLogMessage(event.data)
        break
      case 'system_message':
        // 处理系统消息
        console.log('系统消息:', event.data)
        break
      case 'info_notification':
        // 处理Info类型的轻量提示
        console.log('Info提示:', event.data)
        ElNotification({
          title: '提示',
          message: event.data.message,
          type: 'info',
          position: 'top-right',
          duration: 3000,
          showClose: true
        })
        break
      case 'error':
        error.value = event.data.message || 'WebSocket错误'
        console.error('WebSocket错误:', event.data)
        break
    }
  }

  // 清理函数
  const clearError = () => {
    error.value = null
  }

  return {
    // 状态
    gameState,
    connected,
    connecting,
    error,
    logMessages,
    
    // 计算属性
    globalState,
    gameData,
    directorPlayers,
    directorPlaces,
    actorPlayer,
    actorPlaces,
    actionResult,
    playerList,
    directorPlaceList,
    actorPlaceList,
    
    // 操作
    connect,
    disconnect,
    updateGameState,
    addLogMessage,
    clearLogMessages,
    sendDirectorAction,
    sendPlayerAction,
    updateWeather,
    setNightTime,
    setDestroyPlaces,
    togglePlaceStatus,
    setPlayerLife,
    setPlayerStrength,
    movePlayer,
    togglePlayerBinding,
    destroyPlace,
    sendBroadcast,
    sendDirectorMessageToPlayer,
    sendBatchAirdrop,
    sendBatchItemDeletion,
    addPlayerItem, // 新增导出
    removePlayerItem, // 新增导出
    clearError
  }
})