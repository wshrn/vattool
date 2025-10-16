import { invoke } from '@tauri-apps/api/tauri'
import type { OfflineKeyValidationResult } from '../types/license'
import type { EnvironmentStatus, MirrorOption } from '../types/env'

export class SqlmapAPI {
  static async validateOfflineKey(): Promise<OfflineKeyValidationResult> {
    return await invoke<OfflineKeyValidationResult>('validate_offline_key')
  }

  static async fetchEnvironmentStatus(): Promise<EnvironmentStatus> {
    return await invoke<EnvironmentStatus>('get_environment_status')
  }

  static async initializePythonEnvironment(mirror: MirrorOption): Promise<EnvironmentStatus> {
    return await invoke<EnvironmentStatus>('initialize_python_environment', {
      mirror
    })
  }
}
