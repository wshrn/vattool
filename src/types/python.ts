export interface StatusItem {
  status: boolean
  message: string
}

export interface PythonStatus {
  pythonExists: StatusItem
  pythonEnvVar: StatusItem
  pythonInPath: StatusItem
  scriptsInPath: StatusItem
  pipMirror: StatusItem
}

export interface InitializationResult {
  steps: Array<{
    name: string
    success: boolean
    message: string
  }>
}
