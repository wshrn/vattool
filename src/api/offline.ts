import { invoke } from '@tauri-apps/api/tauri'
import type { OfflineKeyValidationResult } from '@/types/offline'
import { isTauriAvailable } from './utils'

type MessageFn = (title: string, options?: { title?: string; type?: 'error' | 'info' | 'success' }) => Promise<void>

const loadMessageApi = async (): Promise<MessageFn | null> => {
  if (!isTauriAvailable()) {
    return null
  }
  try {
    const { message } = await import('@tauri-apps/api/dialog')
    return message
  } catch (error) {
    console.error('无法加载提示框 API', error)
    return null
  }
}

const loadProcessApi = async () => {
  if (!isTauriAvailable()) {
    return null
  }
  try {
    const { exit } = await import('@tauri-apps/api/process')
    return exit
  } catch (error) {
    console.error('无法加载进程 API', error)
    return null
  }
}

export const validateOfflineKey = async (): Promise<OfflineKeyValidationResult | null> => {
  if (!isTauriAvailable()) {
    return null
  }
  return invoke<OfflineKeyValidationResult>('validate_offline_key')
}

export const ensureOfflineLicense = async (): Promise<boolean> => {
  if (!isTauriAvailable()) {
    return true
  }

  try {
    const result = await validateOfflineKey()
    if (!result?.isValid) {
      const reason = result?.reason ?? '离线密钥无效'
      const expiresAt = result?.expiresAt ? `\n到期时间：${result.expiresAt}` : ''
      console.error(`离线密钥校验失败：${reason}${expiresAt}`)
      const messageFn = await loadMessageApi()
      if (messageFn) {
        await messageFn('认证失败', { title: '认证失败', type: 'error' })
      }
      const exit = await loadProcessApi()
      if (exit) {
        await exit(0)
      }
      return false
    }
    return true
  } catch (error) {
    const detail = error instanceof Error ? error.message : String(error)
    console.error(`离线密钥校验异常：${detail}`)
    const messageFn = await loadMessageApi()
    if (messageFn) {
      await messageFn('认证失败', { title: '认证失败', type: 'error' })
    }
    const exit = await loadProcessApi()
    if (exit) {
      await exit(0)
    }
    return false
  }
}
