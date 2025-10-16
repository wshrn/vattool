export type StatusLevel = 'success' | 'warning' | 'error' | 'info'

export interface StatusItem {
  id: string
  label: string
  ok: boolean
  message: string
  level: StatusLevel
}

export interface MirrorOption {
  label: string
  value: string
  description?: string
}

export interface EnvironmentStatus {
  pythonExecutableFound: boolean
  pythonEnvVariableReady: boolean
  pythonInPath: boolean
  scriptsInPath: boolean
  pipMirror: string | null
  availableMirrors: MirrorOption[]
}
