import { invoke } from '@tauri-apps/api/tauri'
import type { PythonEnvironmentStatus, PythonInitResult } from '@/types/python'
import { isTauriAvailable } from './utils'

export const fetchPythonStatus = async (): Promise<PythonEnvironmentStatus | null> => {
  if (!isTauriAvailable()) {
    return null
  }
  return invoke<PythonEnvironmentStatus>('python_environment_status')
}

export const initializePythonEnvironment = async (mirror: string): Promise<PythonInitResult | null> => {
  if (!isTauriAvailable()) {
    return null
  }
  return invoke<PythonInitResult>('initialize_python_environment', { mirror })
}
