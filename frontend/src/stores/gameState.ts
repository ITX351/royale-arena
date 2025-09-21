import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { 
  DirectorGameState, 
  GlobalState, 
  GameData, 
  Player, 
  Place, 
  ActionResult 
} from '@/types/directorGameState'
import { webSocketService, type WebSocketEvent } from '@/services/webSocketService'

export const useGameStateStore = defineStore('gameState', () => {
  // 状态
  const gameState = ref<DirectorGameState | null>(null)
  const connected = ref(false)
  const connecting = ref(false)
  const error = ref<string | null>(null)
  const logMessages = ref<ActionResult[]>([])
  const maxLogMessages = ref(100) // 最多保存100条日志消息

  // 计算属性
  const globalState = computed<GlobalState | null>(() => {
    return gameState.value?.global_state || null
  })

  const gameData = computed<GameData | null>(() => {
    return gameState.value?.game_data || null
  })

  const players = computed<Record<string, Player>>(() => {
    return gameState.value?.game_data.players || {}
  })

  const places = computed<Record<string, Place>>(() => {
    return gameState.value?.game_data.places || {}
  })

  const actionResult = computed<ActionResult | null>(() => {
    return gameState.value?.action_result || null
  })

  const playerList = computed<Player[]>(() => {
    return Object.values(players.value)
  })

  const placeList = computed<Place[]>(() => {
    return Object.values(places.value)
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

  const updateGameState = (newState: DirectorGameState) => {
    gameState.value = newState
    
    // 如果有动作结果，添加到日志消息中
    if (newState.action_result) {
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

  // 玩家生命值调整
  const updatePlayerLife = (playerId: string, change: number) => {
    sendDirectorAction('life', { player_id: playerId, life_change: change })
  }

  // 玩家体力值调整
  const updatePlayerStrength = (playerId: string, change: number) => {
    sendDirectorAction('strength', { player_id: playerId, strength_change: change })
  }

  // 玩家移动
  const movePlayer = (playerId: string, targetPlace: string) => {
    sendDirectorAction('move_player', { player_id: playerId, target_place: targetPlace })
  }

  // 玩家捆绑/松绑
  const togglePlayerBinding = (playerId: string) => {
    // 先获取玩家当前状态来决定是捆绑还是松绑
    const player = players.value[playerId]
    if (player) {
      if (player.is_bound) {
        // 松绑
        sendDirectorAction('rope', { 
          player_id: playerId, 
          action_type: 'unrope' 
        })
      } else {
        // 捆绑
        sendDirectorAction('rope', { 
          player_id: playerId, 
          action_type: 'rope' 
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
    sendDirectorAction('director_message_to_player', { player_id: playerId, message })
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
    players,
    places,
    actionResult,
    playerList,
    placeList,
    
    // 操作
    connect,
    disconnect,
    updateGameState,
    addLogMessage,
    clearLogMessages,
    sendDirectorAction,
    updateWeather,
    setNightTime,
    setDestroyPlaces,
    togglePlaceStatus,
    updatePlayerLife,
    updatePlayerStrength,
    movePlayer,
    togglePlayerBinding,
    destroyPlace,
    sendBroadcast,
    sendDirectorMessageToPlayer,
    clearError
  }
})