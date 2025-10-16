import { invoke } from '@tauri-apps/api/tauri'

export type StatusLevel = 'success' | 'warning' | 'error' | 'info'

export interface StatusLine {
  label: string
  message: string
  level: StatusLevel
}

export interface PythonEnvironmentStatus {
  statusLines: StatusLine[]
  availableMirrors: PythonMirror[]
  selectedMirror: string
  canInitialize: boolean
  runningOnWindows: boolean
}

export interface PythonMirror {
  id: string
  label: string
  indexUrl: string
}

export interface InitializationResult {
  success: boolean
  message: string
  status?: PythonEnvironmentStatus
}

export const fetchPythonEnvironmentStatus = async (): Promise<PythonEnvironmentStatus> => {
  return invoke<PythonEnvironmentStatus>('get_python_environment_status')
}

export const initializePythonEnvironment = async (mirrorId: string): Promise<InitializationResult> => {
  return invoke<InitializationResult>('initialize_python_environment', { mirrorId })
}
