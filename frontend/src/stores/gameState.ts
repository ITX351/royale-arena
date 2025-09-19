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
  const connect = async (gameId: string, password: string) => {
    connecting.value = true
    error.value = null
    
    try {
      // 添加WebSocket事件监听器
      webSocketService.addEventListener(handleWebSocketEvent)
      
      // 连接到WebSocket
      await webSocketService.connect(gameId, password)
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

  const updateWeather = (weather: number) => {
    sendDirectorAction('weather', { value: weather })
  }

  const setNightTime = (startTime: string | null, endTime: string | null) => {
    sendDirectorAction('set_time', { 
      night_start_time: startTime, 
      night_end_time: endTime 
    })
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
      case 'game_state':
        updateGameState(event.data)
        // TODO: addLogMessage(event.data)
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
    destroyPlace,
    sendBroadcast,
    sendDirectorMessageToPlayer,
    clearError
  }
})