/**
 * 格式化时间戳为本地时间字符串
 * @param timestamp 时间戳字符串
 * @returns 格式化后的时间字符串
 */
export const formatTimestamp = (timestamp: string): string => {
  return new Date(timestamp).toLocaleString('zh-CN')
}

/**
 * 格式化游戏状态显示文本
 * @param status 游戏状态
 * @returns 格式化后的状态文本
 */
export const formatGameStatus = (status: string): string => {
  const statusMap: Record<string, string> = {
    'waiting': '等待中',
    'running': '进行中',
    'paused': '已暂停',
    'ended': '已结束',
    'hidden': '已隐藏',
    'deleted': '已删除'
  }
  return statusMap[status] || status
}

/**
 * 获取游戏状态对应的标签类型
 * @param status 游戏状态
 * @returns Element Plus标签类型
 */
export const getStatusTagType = (status: string): 'success' | 'info' | 'warning' | 'danger' => {
  const typeMap: Record<string, 'success' | 'info' | 'warning' | 'danger'> = {
    'waiting': 'info',
    'running': 'success',
    'paused': 'info',
    'ended': 'warning',
    'hidden': 'info',
    'deleted': 'danger'
  }
  return typeMap[status] || 'info'
}

/**
 * 判断是否显示开始游戏按钮
 * @param status 游戏状态
 * @returns 是否显示开始游戏按钮
 */
export const shouldShowStartButton = (status: string): boolean => {
  return status === 'waiting'
}

/**
 * 判断是否显示暂停游戏按钮
 * @param status 游戏状态
 * @returns 是否显示暂停游戏按钮
 */
export const shouldShowPauseButton = (status: string): boolean => {
  return status === 'running'
}

/**
 * 判断是否显示继续游戏按钮
 * @param status 游戏状态
 * @returns 是否显示继续游戏按钮
 */
export const shouldShowResumeButton = (status: string): boolean => {
  return status === 'paused'
}

/**
 * 判断是否显示结束游戏按钮
 * @param status 游戏状态
 * @returns 是否显示结束游戏按钮
 */
export const shouldShowEndButton = (status: string): boolean => {
  return status === 'running' || status === 'paused'
}