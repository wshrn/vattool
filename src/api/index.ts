import { invoke } from '@tauri-apps/api/tauri'

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

export interface EnvironmentStatus {
  pythonFound: boolean
  pythonPath?: string
  python3EnvExists: boolean
  python3EnvValue?: string
  pathHasPython: boolean
  pathHasScripts: boolean
  pipConfigured: boolean
  pipIndexUrl?: string
  messages: string[]
}

export interface InitializationResult {
  success: boolean
  message: string
  status?: EnvironmentStatus
}

export type ThemeMode = 'light' | 'dark' | 'auto'

export const isTauriAvailable = () =>
  typeof window !== 'undefined' && Boolean((window as any).__TAURI__)

export class ToolboxAPI {
  static async validateOfflineKey(): Promise<OfflineKeyValidationResult> {
    return await invoke<OfflineKeyValidationResult>('validate_offline_key')
  }

  static async getEnvironmentStatus(): Promise<EnvironmentStatus> {
    return await invoke<EnvironmentStatus>('get_environment_status')
  }

  static async initializeEnvironment(mirror: string): Promise<InitializationResult> {
    return await invoke<InitializationResult>('initialize_environment', { mirror })
  }

  static async readTheme(): Promise<ThemeMode> {
    return await invoke<ThemeMode>('tool_read_theme')
  }
}
