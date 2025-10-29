import { ElMessage } from 'element-plus'
import type { DirectorGameState } from '@/types/gameStateTypes'
import { API_CONFIG } from './config'

// WebSocket连接状态常量
export const WebSocketStatus = {
  CONNECTING: 'connecting',
  CONNECTED: 'connected',
  DISCONNECTED: 'disconnected',
  ERROR: 'error'
} as const;

export type WebSocketStatus = typeof WebSocketStatus[keyof typeof WebSocketStatus];

// WebSocket事件类型
export interface WebSocketEvent {
  type: 'state_update' | 'system_message' | 'action_result' | 'error' | 'info_notification'
  data: any
  timestamp: Date
}

// WebSocket服务类
export class WebSocketService {
  private ws: WebSocket | null = null
  private url: string = ''
  private reconnectAttempts: number = 0
  private maxReconnectAttempts: number = 5
  private reconnectInterval: number = 3000
  private status: WebSocketStatus = WebSocketStatus.DISCONNECTED
  private listeners: Array<(event: WebSocketEvent) => void> = []
  private gameId: string = ''
  private password: string = ''
  private userType: string = '' // 默认为导演

  // 连接到WebSocket服务器
  connect(gameId: string, password: string, userType: string, timeout: number = API_CONFIG.TIMEOUT): Promise<void> {
    return new Promise((resolve, reject) => {
      // 如果已经连接，先断开
      if (this.ws) {
        this.disconnect()
      }

      this.gameId = gameId
      this.password = password
      this.userType = userType // 保存用户类型

      try {
        this.url = this.buildWebSocketUrl(gameId, password, userType)
      } catch (error) {
        console.error('构建WebSocket连接地址失败:', error)
        this.setStatus(WebSocketStatus.ERROR)
        reject(error)
        return
      }

      console.log('连接URL:', this.url)
      
      // 设置连接超时定时器
      const timeoutId = setTimeout(() => {
        if (this.ws && this.ws.readyState === WebSocket.CONNECTING) {
          console.warn('WebSocket连接超时')
          this.ws.close() // 关闭连接
          this.setStatus(WebSocketStatus.ERROR)
          reject(new Error('连接超时'))
        }
      }, timeout)

      try {
        this.setStatus(WebSocketStatus.CONNECTING)
        this.ws = new WebSocket(this.url)

        this.ws.onopen = () => {
          clearTimeout(timeoutId) // 清除超时定时器
          console.log('WebSocket连接已建立')
          this.setStatus(WebSocketStatus.CONNECTED)
          this.reconnectAttempts = 0
          resolve()
        }

        this.ws.onmessage = (event) => {
          try {
            const data = JSON.parse(event.data)
            this.handleMessage(data)
          } catch (error) {
            console.error('解析WebSocket消息失败:', error)
            this.emitEvent({
              type: 'error',
              data: { message: '解析消息失败', error },
              timestamp: new Date()
            })
          }
        }

        this.ws.onclose = () => {
          clearTimeout(timeoutId) // 清除超时定时器
          console.log('WebSocket连接已关闭')
          this.setStatus(WebSocketStatus.DISCONNECTED)
          
          // 如果不是主动断开，尝试重连
          if (this.status !== WebSocketStatus.DISCONNECTED) {
            this.attemptReconnect()
          }
        }

        this.ws.onerror = (error) => {
          clearTimeout(timeoutId) // 清除超时定时器
          console.error('WebSocket连接错误:', error)
          this.setStatus(WebSocketStatus.ERROR)
          this.emitEvent({
            type: 'error',
            data: { message: '连接错误', error },
            timestamp: new Date()
          })
          reject(error)
        }
      } catch (error) {
        clearTimeout(timeoutId) // 清除超时定时器
        console.error('创建WebSocket连接失败:', error)
        this.setStatus(WebSocketStatus.ERROR)
        reject(error)
      }
    })
  }

  // 断开WebSocket连接
  disconnect(): void {
    if (this.ws) {
      this.ws.close()
      this.ws = null
    }
    this.setStatus(WebSocketStatus.DISCONNECTED)
  }

  // 发送消息到服务器
  sendMessage(message: any): void {
    if (this.ws && this.status === WebSocketStatus.CONNECTED) {
      try {
        console.log('发送WebSocket消息:', message)
        this.ws.send(JSON.stringify(message))
      } catch (error) {
        console.error('发送WebSocket消息失败:', error)
        ElMessage.error('发送消息失败')
      }
    } else {
      console.warn('WebSocket未连接，无法发送消息')
      ElMessage.warning('连接未建立，无法发送消息')
    }
  }

  // 发送导演控制指令
  sendDirectorAction(action: string, params: Record<string, any> = {}): void {
    const message = {
      type: 'director_action',
      data: {
        action,
        ...params
      }
    }
    this.sendMessage(message)
  }

  // 添加事件监听器
  addEventListener(listener: (event: WebSocketEvent) => void): void {
    this.listeners.push(listener)
  }

  // 移除事件监听器
  removeEventListener(listener: (event: WebSocketEvent) => void): void {
    const index = this.listeners.indexOf(listener)
    if (index > -1) {
      this.listeners.splice(index, 1)
    }
  }

  // 获取当前连接状态
  getStatus(): WebSocketStatus {
    return this.status
  }

  // 重新连接
  private attemptReconnect(): void {
    if (this.reconnectAttempts < this.maxReconnectAttempts) {
      this.reconnectAttempts++
      console.log(`尝试重连 (${this.reconnectAttempts}/${this.maxReconnectAttempts})...`)
      ElMessage.warning(`连接断开，正在尝试重连 (${this.reconnectAttempts}/${this.maxReconnectAttempts})...`)

      setTimeout(() => {
        this.connect(this.gameId, this.password, this.userType).catch(() => {
          // 重连失败，继续尝试
          this.attemptReconnect()
        })
      }, this.reconnectInterval)
    } else {
      console.error('达到最大重连次数，停止重连')
      ElMessage.error('连接失败，请刷新页面重试')
    }
  }

  // 处理接收到的消息
  private handleMessage(data: any): void {
    console.log('收到WebSocket消息:', data);
    
    // 根据消息类型进行处理
    switch (data.type) {
      case 'system_message':
        // 系统消息
        this.emitEvent({
          type: 'system_message',
          data: data.data,
          timestamp: new Date()
        });
        break;
        
      case 'error': {
        const message = data?.data?.message ?? '连接发生错误，请稍后重试';
        // ElMessage.error(message);
        this.emitEvent({
          type: 'error',
          data: { message, raw: data.data },
          timestamp: new Date()
        });
        break;
      }

      case 'game_state': {
        // 游戏状态更新消息
        const gameState: DirectorGameState = {
          global_state: data.data.global_state,
          game_data: data.data.game_data,
          action_result: data.data.action_result || null
        };
        
        this.emitEvent({
          type: 'state_update',
          data: gameState,
          timestamp: new Date()
        });
        
        // 如果有动作结果，也发送动作结果事件
        if (data.data.action_result) {
          // 检查消息类型，如果是Info类型，发送轻量提示事件
          if (data.data.action_result.message_type === 'Info') {
            this.emitEvent({
              type: 'info_notification',
              data: {
                message: data.data.action_result.log_message,
                timestamp: data.data.action_result.timestamp
              },
              timestamp: new Date()
            });
          } else {
            // 其他类型的消息使用原有的action_result事件
            this.emitEvent({
              type: 'action_result',
              data: data.data.action_result,
              timestamp: new Date()
            });
          }
        }
        break;
      }
      default:
        // 未知消息类型
        console.warn('未知的WebSocket消息类型:', data);
        this.emitEvent({
          type: 'error',
          data: { message: '未知消息类型', data },
          timestamp: new Date()
        });
        break;
    }
  }

  // 设置连接状态
  private setStatus(status: WebSocketStatus): void {
    this.status = status
  }

  // 触发事件
  private emitEvent(event: WebSocketEvent): void {
    // 使用slice创建副本，防止在遍历过程中数组被修改
    const listenersCopy = this.listeners.slice()
    listenersCopy.forEach(listener => {
      try {
        listener(event)
      } catch (error) {
        console.error('事件监听器执行出错:', error)
      }
    })
  }

  private buildWebSocketUrl(gameId: string, password: string, userType: string): string {
    const resolvedBase = new URL(API_CONFIG.BASE_URL, window.location.origin)
    const wsUrl = new URL(resolvedBase.toString())
    wsUrl.protocol = resolvedBase.protocol === 'https:' ? 'wss:' : 'ws:'
    wsUrl.pathname = `${resolvedBase.pathname.replace(/\/+$/, '')}/ws/${encodeURIComponent(gameId)}`
    wsUrl.search = ''
    wsUrl.searchParams.set('user_type', userType)
    wsUrl.searchParams.set('password', password)
    return wsUrl.toString()
  }
}

// 创建单例实例
export const webSocketService = new WebSocketService()