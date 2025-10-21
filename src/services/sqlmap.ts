import { invoke } from '@tauri-apps/api/core'
import { isTauri } from '../utils/runtime'
export interface OfflineLicensePayload {
  userId: number
  username: string
  email: string
  deviceId: string
  expiresAt: string
  issuedAt: string
}

export interface OfflineKeyValidationResult {
  isValid: boolean
  reason?: string
  expiresAt?: string
  payload?: OfflineLicensePayload
}

const tauriMessage = async (content: string, options: { title: string; type: 'info' | 'error' | 'warning' | 'success' }) => {
  if (!isTauri()) {
    alert(`${options.title}: ${content}`)
    return
  }
  const dialog = (window as any).__TAURI__?.dialog
  if (dialog?.message) {
    await dialog.message(content, options)
  }
}

const tauriExit = async (code: number) => {
  if (!isTauri()) {
    return
  }
  const processApi = (window as any).__TAURI__?.process
  if (processApi?.exit) {
    await processApi.exit(code)
  }
}

export class SqlmapAPI {
  static async validateOfflineKey(): Promise<OfflineKeyValidationResult> {
    return await invoke<OfflineKeyValidationResult>('validate_offline_key')
  }

  static async getDeviceId(): Promise<string> {
    return await invoke<string>('get_device_id')
  }
}

export const ensureOfflineLicense = async (): Promise<boolean> => {
  if (!isTauri()) {
    return true
  }

  try {
    const result = await SqlmapAPI.validateOfflineKey()
    if (!result.isValid) {
      const reason = result.reason ?? '离线密钥无效'
      const expiresAt = result.expiresAt ? `\n到期时间：${result.expiresAt}` : ''
      console.error(`离线密钥校验失败：${reason}${expiresAt}`)
      await tauriMessage(`认证失败：${reason}${expiresAt}`, { title: '认证失败', type: 'error' })
      await tauriExit(0)
      return false
    }
    return true
  } catch (error) {
    const detail = error instanceof Error ? error.message : String(error)
    console.error(`离线密钥校验失败：${detail}`)
    await tauriMessage(`认证失败：${detail}`, { title: '认证失败', type: 'error' })
    await tauriExit(0)
    return false
  }
}
