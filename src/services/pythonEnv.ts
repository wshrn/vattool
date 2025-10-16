import { invoke } from '@tauri-apps/api/core'

export interface StatusLine {
  ok: boolean
  message: string
  detail?: string
}

export interface PythonEnvStatus {
  pythonPresent: StatusLine
  pythonEnvVar: StatusLine
  pathConfigured: StatusLine
  scriptsConfigured: StatusLine
  pipMirrorConfigured: StatusLine & { currentMirror?: string }
  pythonPath?: string
  scriptsPath?: string
  mirrorCandidates: MirrorOption[]
}

export interface MirrorOption {
  label: string
  value: string
  trustedHosts: string[]
}

export interface InitializePayload {
  mirror: string
}

export const fetchPythonEnvStatus = async (): Promise<PythonEnvStatus> => {
  return await invoke<PythonEnvStatus>('get_python_environment_status')
}

export const initializePythonEnvironment = async (
  payload: InitializePayload,
): Promise<PythonEnvStatus> => {
  return await invoke<PythonEnvStatus>('initialize_python_environment', payload)
}
