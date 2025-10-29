/**
 * 公用功能模块
 * 存放项目中通用的工具函数
 */

/**
 * 比较两个字符串数组是否相等
 * @param incoming 待比较的数组
 * @param current 基准数组
 * @returns 两个数组是否相等
 */
export function areStringArraysEqual(
  incoming: string[] | undefined | null,
  current: string[]
): boolean {
  const normalizedIncoming = Array.isArray(incoming) ? incoming : []
  if (normalizedIncoming.length !== current.length) {
    return false
  }
  for (let index = 0; index < normalizedIncoming.length; index += 1) {
    if (normalizedIncoming[index] !== current[index]) {
      return false
    }
  }
  return true
}

/**
 * 统一获取带 base 路径的 URL
 * @param path 要拼接的路径
 * @returns 完整的URL路径
 */
export function getBasePathUrl(path: string): string {
  const basePath = import.meta.env.BASE_URL || '/'
  const normalizedBasePath = basePath.endsWith('/') ? basePath : basePath + '/'
  return normalizedBasePath + path
}