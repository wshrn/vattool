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

export interface PythonEnvStatusItem {
  key: string
  label: string
  ok: boolean
  detail?: string
  level: 'success' | 'warning' | 'error' | 'info'
}

export interface PythonEnvStatusResponse {
  items: PythonEnvStatusItem[]
  mirrorSources: MirrorSource[]
  selectedMirror: string
}

export interface MirrorSource {
  key: string
  label: string
  url: string
  description: string
}

export interface InitializationResult {
  success: boolean
  message: string
  status?: PythonEnvStatusResponse
}

export class SqlmapAPI {
  static async validateOfflineKey(): Promise<OfflineKeyValidationResult> {
    return invoke<OfflineKeyValidationResult>('validate_offline_key')
  }
}

export class PythonEnvAPI {
  static async fetchStatus(): Promise<PythonEnvStatusResponse> {
    return invoke<PythonEnvStatusResponse>('python_env_status')
  }

  static async initializeEnvironment(mirrorKey: string): Promise<InitializationResult> {
    return invoke<InitializationResult>('initialize_python_environment', { mirrorKey })
  }
}
