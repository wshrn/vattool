import { invoke } from '@tauri-apps/api/tauri'
import { message } from '@tauri-apps/api/dialog'
import { exit } from '@tauri-apps/api/process'

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

const isTauriAvailable = () => typeof window !== 'undefined' && Boolean((window as any).__TAURI__)

export const ensureOfflineLicense = async (): Promise<boolean> => {
  if (!isTauriAvailable()) {
    return true
  }

  try {
    const result = await invoke<OfflineKeyValidationResult>('validate_offline_key')
    if (!result.isValid) {
      const reason = result.reason ?? '离线密钥无效'
      const expiresAt = result.expiresAt ? `\n到期时间：${result.expiresAt}` : ''
      console.error(`离线密钥校验失败：${reason}${expiresAt}`)
      await message(`离线密钥校验失败：${reason}${expiresAt}`, { title: '认证失败', type: 'error' })
      await exit(0)
      return false
    }
    return true
  } catch (error) {
    const detail = error instanceof Error ? error.message : String(error)
    console.error(`离线密钥校验失败：${detail}`)
    await message(`离线密钥校验失败：${detail}`, { title: '认证失败', type: 'error' })
    await exit(0)
    return false
  }
}
