<template>
  <div class="w-screen h-screen flex flex-col bg-[var(--app-surface-bg)] text-gray-800 dark:text-gray-100">
    <TitleBar />
    <main class="flex-1 overflow-y-auto p-6 lg:p-10 bg-gray-50 dark:bg-gray-900 scrollbar-thin">
      <div class="max-w-3xl mx-auto space-y-6">
        <header class="space-y-2">
          <h1 class="text-2xl font-semibold text-gray-900 dark:text-gray-100">
            Python3 环境初始化助手
          </h1>
          <p class="text-sm text-gray-500 dark:text-gray-400">
            工具会自动检测当前目录下的 Python 环境，并提供一键初始化国内镜像源与环境变量的能力。
          </p>
        </header>

        <section class="card-apple p-6 space-y-4 animate-slide-up">
          <h2 class="text-lg font-medium text-gray-900 dark:text-gray-100">当前检测状态</h2>
          <ul class="space-y-3">
            <li
              v-for="item in statusItems"
              :key="item.id"
              class="result-item"
              :class="statusClass(item)"
            >
              <p class="font-medium">{{ item.label }}</p>
              <p class="text-sm text-gray-500 dark:text-gray-400">{{ item.message }}</p>
            </li>
          </ul>

          <div class="space-y-2">
            <label class="text-sm font-medium text-gray-700 dark:text-gray-300">
              全局 PIP 国内镜像源
            </label>
            <select
              v-model="selectedMirror"
              class="input-apple"
            >
              <option
                v-for="mirror in mirrors"
                :key="mirror.value"
                :value="mirror.value"
              >
                {{ mirror.label }} ({{ mirror.value }})
              </option>
            </select>
            <p class="text-xs text-gray-400">
              默认选择清华大学开源镜像站，可根据需要切换为阿里云、腾讯云等常见国内源。
            </p>
          </div>

          <div class="flex items-center justify-between">
            <p class="text-sm text-gray-500 dark:text-gray-400">
              完成后点击按钮自动设置环境变量和国内镜像源。
            </p>
            <button
              class="btn-primary flex items-center space-x-2"
              :disabled="loading"
              @click="initializeEnvironment"
            >
              <svg
                v-if="loading"
                class="w-4 h-4 text-white loading-spinner"
                viewBox="0 0 24 24"
                fill="none"
                xmlns="http://www.w3.org/2000/svg"
              >
                <path
                  d="M12 2a10 10 0 1 0 10 10"
                  stroke="currentColor"
                  stroke-width="2"
                  stroke-linecap="round"
                />
              </svg>
              <span>{{ loading ? '正在初始化...' : '开始初始化' }}</span>
            </button>
          </div>
        </section>
      </div>
    </main>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import TitleBar from './components/TitleBar.vue'
import { SqlmapAPI } from './utils/api'
import { isTauriAvailable } from './utils/tauri'
import type { EnvironmentStatus, MirrorOption, StatusItem } from './types/env'

const status = ref<EnvironmentStatus | null>(null)
const loading = ref(false)
const mirrors = ref<MirrorOption[]>([])
const selectedMirror = ref<string>('https://pypi.tuna.tsinghua.edu.cn/simple')

const buildStatusItem = (id: string, ok: boolean, successMessage: string, failureMessage: string): StatusItem => ({
  id,
  label: successLabels[id] ?? id,
  ok,
  message: ok ? successMessage : failureMessage,
  level: ok ? 'success' : 'warning'
})

const successLabels: Record<string, string> = {
  pythonExecutableFound: 'Python 可执行文件检测',
  pythonEnvVariableReady: 'python3 环境变量配置',
  pythonInPath: 'PATH 中的 Python 配置',
  scriptsInPath: 'PATH 中的 Scripts 目录',
  pipMirror: 'PIP 国内镜像源配置'
}

const statusItems = computed<StatusItem[]>(() => {
  if (!status.value) {
    return []
  }

  return [
    buildStatusItem(
      'pythonExecutableFound',
      status.value.pythonExecutableFound,
      '已检测到当前目录存在 Python 可执行文件。',
      '未检测到 Python 可执行文件，请确认工具与 python.exe 同目录。'
    ),
    buildStatusItem(
      'pythonEnvVariableReady',
      status.value.pythonEnvVariableReady,
      '环境变量 python3 已正确指向 Python 安装目录。',
      '未发现名为 python3 的环境变量，初始化时将自动创建。'
    ),
    buildStatusItem(
      'pythonInPath',
      status.value.pythonInPath,
      'PATH 中已包含 Python 可执行目录。',
      'PATH 中缺少 Python，可执行初始化后添加。'
    ),
    buildStatusItem(
      'scriptsInPath',
      status.value.scriptsInPath,
      'PATH 中已包含 Scripts 目录。',
      'PATH 中缺少 Scripts 目录，将在初始化时自动补全。'
    ),
    {
      id: 'pipMirror',
      label: successLabels.pipMirror,
      ok: Boolean(status.value.pipMirror),
      message: status.value.pipMirror
        ? `已配置全局镜像：${status.value.pipMirror}`
        : '未配置全局镜像，将使用所选源进行初始化。',
      level: status.value.pipMirror ? 'success' : 'warning'
    }
  ]
})

const statusClass = (item: StatusItem) => {
  if (item.level === 'success') {
    return 'success'
  }
  if (item.level === 'error') {
    return 'error'
  }
  if (item.level === 'info') {
    return 'info'
  }
  return 'warning'
}

const loadStatus = async () => {
  if (!isTauriAvailable()) {
    status.value = {
      pythonExecutableFound: true,
      pythonEnvVariableReady: false,
      pythonInPath: false,
      scriptsInPath: false,
      pipMirror: null,
      availableMirrors: defaultMirrors
    }
    mirrors.value = defaultMirrors
    selectedMirror.value = defaultMirrors[0]?.value ?? selectedMirror.value
    return
  }

  try {
    const result = await SqlmapAPI.fetchEnvironmentStatus()
    status.value = result
    mirrors.value = result.availableMirrors
    if (result.pipMirror) {
      selectedMirror.value = result.pipMirror
    } else if (result.availableMirrors.length > 0) {
      selectedMirror.value = result.availableMirrors[0].value
    }
  } catch (error) {
    console.error('加载环境状态失败', error)
  }
}

const initializeEnvironment = async () => {
  if (!status.value) {
    return
  }
  loading.value = true
  try {
    let result: EnvironmentStatus
    if (isTauriAvailable()) {
      const mirror = mirrors.value.find(item => item.value === selectedMirror.value)
      if (!mirror) {
        throw new Error('请选择有效的镜像源')
      }
      result = await SqlmapAPI.initializePythonEnvironment(mirror)
    } else {
      result = {
        ...status.value,
        pythonEnvVariableReady: true,
        pythonInPath: true,
        scriptsInPath: true,
        pipMirror: selectedMirror.value
      }
    }
    status.value = result
  } catch (error) {
    console.error('初始化失败', error)
  } finally {
    loading.value = false
  }
}

const defaultMirrors: MirrorOption[] = [
  { label: '清华大学', value: 'https://pypi.tuna.tsinghua.edu.cn/simple' },
  { label: '阿里云', value: 'https://mirrors.aliyun.com/pypi/simple/' },
  { label: '腾讯云', value: 'https://mirrors.cloud.tencent.com/pypi/simple' },
  { label: '华为云', value: 'https://repo.huaweicloud.com/repository/pypi/simple' }
]

onMounted(() => {
  mirrors.value = defaultMirrors
  selectedMirror.value = defaultMirrors[0].value
  void loadStatus()
})
</script>
