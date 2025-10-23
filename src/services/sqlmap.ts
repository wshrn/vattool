import { invoke } from '@tauri-apps/api/core'

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

const isTauriAvailable = () =>
  typeof window !== 'undefined' && Boolean((window as any).__TAURI__)

const showAuthError = async (content: string) => {
  if (!isTauriAvailable()) {
    alert(content)
    return
  }

  const dialog = (window as any).__TAURI__?.dialog
  if (dialog?.message) {
    await dialog.message(content, { title: '认证失败', type: 'error' })
  } else {
    alert(content)
  }
}

const exitApp = async () => {
  if (!isTauriAvailable()) {
    return
  }
  const processApi = (window as any).__TAURI__?.process
  if (processApi?.exit) {
    await processApi.exit(0)
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
  if (!isTauriAvailable()) {
    return true
  }

  try {
    const result = await SqlmapAPI.validateOfflineKey()
    if (!result.isValid) {
      const reason = result.reason ?? '离线密钥无效'
      const expiresAt = result.expiresAt ? `\n到期时间：${result.expiresAt}` : ''
      const message = `认证失败：${reason}${expiresAt}`
      console.error(`离线密钥校验失败：${reason}${expiresAt}`)
      await showAuthError(message)
      await exitApp()
      return false
    }
    return true
  } catch (error) {
    const detail = error instanceof Error ? error.message : String(error)
    const message = `认证失败：${detail}`
    console.error(`离线密钥校验失败：${detail}`)
    await showAuthError(message)
    await exitApp()
    return false
  }
}
