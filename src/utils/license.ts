import { message } from '@tauri-apps/api/dialog'
import { exit } from '@tauri-apps/api/process'
import type { OfflineKeyValidationResult } from '../types/license'
import { SqlmapAPI } from './api'
import { isTauriAvailable } from './tauri'

export const ensureOfflineLicense = async (): Promise<boolean> => {
  if (!isTauriAvailable()) {
    return true
  }

  try {
    const result: OfflineKeyValidationResult = await SqlmapAPI.validateOfflineKey()
    if (!result.isValid) {
      const reason = result.reason ?? '离线密钥无效'
      const expiresAt = result.expiresAt ? `\n到期时间：${result.expiresAt}` : ''
      console.error(`离线密钥校验失败：${reason}${expiresAt}`)
      await message('认证失败', { title: '认证失败', type: 'error' })
      await exit(0)
      return false
    }
    return true
  } catch (error) {
    const detail = error instanceof Error ? error.message : String(error)
    console.error(`离线密钥校验失败：${detail}`)
    await message('认证失败', { title: '认证失败', type: 'error' })
    await exit(0)
    return false
  }
}
