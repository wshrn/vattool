export interface PythonEnvironmentStatus {
  pythonExists: boolean
  pythonEnvVarSet: boolean
  pythonPathSet: boolean
  scriptsPathSet: boolean
  mirrorConfigured: boolean
  mirrorName: string
  pythonPath?: string
  scriptsPath?: string
  details?: string
}

export interface PythonInitResult {
  success: boolean
  message: string
  status: PythonEnvironmentStatus
}
