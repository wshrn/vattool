import { invoke } from '@tauri-apps/api/tauri'

export interface PythonEnvironmentStatus {
  pythonExecutableFound: boolean
  pythonExecutablePath?: string
  pythonEnvVarSet: boolean
  pythonEnvVarValue?: string
  pythonPathConfigured: boolean
  scriptsPathConfigured: boolean
  pipMirrorConfigured: boolean
  pipMirrorUrl?: string
}

export type MirrorOptionKey = 'tsinghua' | 'aliyun' | 'douban' | 'huawei' | 'pypi'

export interface InitializePythonOptions {
  mirror: MirrorOptionKey
}

export const fetchPythonStatus = async (): Promise<PythonEnvironmentStatus> => {
  return invoke<PythonEnvironmentStatus>('get_python_env_status')
}

export const initializePythonEnvironment = async (options: InitializePythonOptions) => {
  return invoke<PythonEnvironmentStatus>('initialize_python_environment', options)
}

export const mirrorOptions: Record<MirrorOptionKey, { label: string; url: string }> = {
  tsinghua: { label: '清华大学', url: 'https://pypi.tuna.tsinghua.edu.cn/simple' },
  aliyun: { label: '阿里云', url: 'https://mirrors.aliyun.com/pypi/simple/' },
  douban: { label: '豆瓣', url: 'https://pypi.doubanio.com/simple/' },
  huawei: { label: '华为云', url: 'https://mirrors.huaweicloud.com/repository/pypi/simple/' },
  pypi: { label: '官方 PyPI', url: 'https://pypi.org/simple' }
}
