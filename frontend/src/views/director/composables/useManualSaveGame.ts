import { ref, type Ref } from 'vue'
import { ElMessage } from 'element-plus'
import { directorService } from '@/services/directorService'

type SuccessMessage = string | ((saveFileName?: string) => string)

type ManualSaveOptions = {
  successMessage?: SuccessMessage
}

type ManualSaveResult = {
  success: boolean
  saveFileName?: string
  errorMessage?: string
}

/**
 * Shared manual save logic for director-facing views.
 * Handles credential validation, API invocation, and user feedback.
 */
export function useManualSaveGame(
  gameId: Ref<string | undefined | null>,
  directorPassword: Ref<string | undefined | null>
) {
  const saving = ref(false)

  const manualSave = async (options?: ManualSaveOptions): Promise<ManualSaveResult> => {
    if (!directorPassword.value) {
      const errorMessage = '缺少导演密码'
      ElMessage.error(errorMessage)
      return { success: false, errorMessage }
    }

    if (!gameId.value) {
      const errorMessage = '未找到有效的游戏信息'
      ElMessage.error(errorMessage)
      return { success: false, errorMessage }
    }

    saving.value = true

    try {
      const response = await directorService.manualSaveGame(
        gameId.value,
        directorPassword.value
      )

      if (!response?.success) {
        throw new Error(response?.message || '存盘失败')
      }

      const saveFileName: string | undefined = response.save_file_name
      const successMessage = resolveSuccessMessage(options?.successMessage, saveFileName)

      if (successMessage) {
        ElMessage.success(successMessage)
      }

      return { success: true, saveFileName }
    } catch (error: any) {
      const errorMessage = error?.response?.status === 401
        ? '导演密码错误'
        : (error?.message || '存盘失败')
      ElMessage.error(errorMessage)
      return { success: false, errorMessage }
    } finally {
      saving.value = false
    }
  }

  return {
    saving,
    manualSave
  }
}

function resolveSuccessMessage(
  customMessage: SuccessMessage | undefined,
  saveFileName?: string
): string {
  if (typeof customMessage === 'function') {
    return customMessage(saveFileName)
  }

  if (typeof customMessage === 'string') {
    return customMessage
  }

  return saveFileName
    ? `游戏状态已保存至: ${saveFileName}`
    : '游戏状态已保存'
}
