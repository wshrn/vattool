<template>
  <div class="h-screen w-screen flex flex-col bg-[var(--app-surface-bg)] text-gray-900 dark:text-gray-100 transition-colors duration-300">
    <TitleBar />
    <main class="flex-1 overflow-y-auto scrollbar-thin">
      <div class="max-w-4xl mx-auto p-6">
        <div class="card-apple p-8 space-y-8 animate-fade-in">
          <header class="flex flex-col md:flex-row md:items-center md:justify-between gap-4">
            <div>
              <h1 class="text-2xl font-semibold text-gray-900 dark:text-gray-100">Python 环境初始化助手</h1>
              <p class="text-gray-500 dark:text-gray-400 mt-1">
                自动检测当前目录下的 Python 安装并完成环境变量与国内源配置。
              </p>
            </div>
            <button
              class="btn-ghost inline-flex items-center gap-2"
              @click="toggleTheme"
              :title="`当前主题：${themeLabel}`"
            >
              <component :is="themeIcon" class="w-5 h-5" />
              <span class="hidden sm:inline">切换主题</span>
            </button>
          </header>

          <section class="space-y-4">
            <StatusRow
              title="Python 可执行文件"
              :item="status?.pythonExists"
              success-text="已检测到当前目录中的 Python 可执行文件"
              failed-text="未找到 python.exe，请将工具放在 Python 安装目录下"
            />
            <StatusRow
              title="python3 变量"
              :item="status?.pythonEnvVar"
              success-text="已设置 python3 用户环境变量"
              failed-text="尚未创建 python3 环境变量"
            />
            <StatusRow
              title="PATH 中的 Python"
              :item="status?.pythonInPath"
              success-text="PATH 已包含 %python3%"
              failed-text="PATH 尚未包含 %python3%，请初始化"
            />
            <StatusRow
              title="PATH 中的 Scripts"
              :item="status?.scriptsInPath"
              success-text="PATH 已包含 %python3%\\Scripts"
              failed-text="PATH 尚未包含 %python3%\\Scripts"
            />
            <StatusRow
              title="PIP 国内源"
              :item="status?.pipMirror"
              success-text="已配置 pip 国内镜像"
              failed-text="尚未配置 pip 国内镜像"
            />
          </section>

          <section class="space-y-3">
            <label class="block text-sm font-medium text-gray-600 dark:text-gray-300" for="mirror">
              选择需要配置的国内镜像
            </label>
            <select
              id="mirror"
              v-model="selectedMirror"
              class="input-apple"
            >
              <option v-for="option in mirrorOptions" :key="option.value" :value="option.value">
                {{ option.label }}
              </option>
            </select>
            <p class="text-xs text-gray-400 dark:text-gray-500">
              初始化将写入 pip 配置文件，确保在所有终端中生效。
            </p>
          </section>

          <button
            class="btn-primary w-full justify-center"
            :class="{ 'opacity-75 pointer-events-none': initializing }"
            @click="handleInitialize"
          >
            <span v-if="initializing" class="flex items-center gap-2">
              <svg class="w-4 h-4 loading-spinner" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
                <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
                <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8v4a4 4 0 00-4 4H4z" />
              </svg>
              正在初始化...
            </span>
            <span v-else class="flex items-center gap-2">
              <RocketLaunchIcon class="w-5 h-5" />
              开始初始化
            </span>
          </button>
        </div>
      </div>
    </main>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import { message } from '@tauri-apps/api/dialog'
import { storeToRefs } from 'pinia'
import { RocketLaunchIcon, MoonIcon, SunIcon, ComputerDesktopIcon } from '@heroicons/vue/24/outline'
import TitleBar from './components/TitleBar.vue'
import StatusRow from './components/StatusRow.vue'
import { useThemeStore } from './stores/theme'
import type { PythonStatus, InitializationResult } from './types/python'

const mirrorOptions = [
  { value: 'tsinghua', label: '清华大学 TUNA 镜像', url: 'https://pypi.tuna.tsinghua.edu.cn/simple', host: 'pypi.tuna.tsinghua.edu.cn' },
  { value: 'aliyun', label: '阿里云镜像', url: 'https://mirrors.aliyun.com/pypi/simple/', host: 'mirrors.aliyun.com' },
  { value: 'huawei', label: '华为云镜像', url: 'https://repo.huaweicloud.com/repository/pypi/simple', host: 'repo.huaweicloud.com' },
  { value: 'douban', label: '豆瓣镜像', url: 'https://pypi.doubanio.com/simple', host: 'pypi.doubanio.com' }
] as const

type MirrorOption = typeof mirrorOptions[number]

const isTauriAvailable = () =>
  typeof window !== 'undefined' && Boolean((window as any).__TAURI__)

const themeStore = useThemeStore()
const { theme, isDark } = storeToRefs(themeStore)

const status = ref<PythonStatus | null>(null)
const initializing = ref(false)
const selectedMirror = ref<MirrorOption['value']>('tsinghua')

const themeLabel = computed(() => {
  switch (theme.value) {
    case 'dark':
      return '深色模式'
    case 'light':
      return '浅色模式'
    default:
      return '跟随系统'
  }
})

const themeIcon = computed(() => {
  if (theme.value === 'auto') {
    return ComputerDesktopIcon
  }
  return isDark.value ? MoonIcon : SunIcon
})

const toggleTheme = () => {
  const modes: Array<typeof theme.value> = ['light', 'dark', 'auto']
  const currentIndex = modes.indexOf(theme.value)
  const next = modes[(currentIndex + 1) % modes.length]
  void themeStore.setTheme(next)
}

const fetchStatus = async () => {
  if (!isTauriAvailable()) {
    return
  }
  try {
    status.value = await invoke<PythonStatus>('fetch_python_status')
  } catch (error) {
    console.error('获取状态失败', error)
  }
}

const describeResult = (result: InitializationResult) => {
  return result.steps
    .map(step => `${step.success ? '✅' : '❌'} ${step.name}：${step.message}`)
    .join('\n')
}

const handleInitialize = async () => {
  if (!isTauriAvailable() || initializing.value) {
    return
  }
  initializing.value = true
  try {
    const result = await invoke<InitializationResult>('initialize_environment', {
      mirror: selectedMirror.value
    })
    const success = result.steps.every(step => step.success)
    await message(describeResult(result), {
      title: success ? '初始化完成' : '初始化存在问题',
      type: success ? 'info' : 'error'
    })
    await fetchStatus()
  } catch (error) {
    const detail = error instanceof Error ? error.message : String(error)
    await message(`初始化失败：${detail}`, { title: '错误', type: 'error' })
  } finally {
    initializing.value = false
  }
}

onMounted(() => {
  void fetchStatus()
})
</script>
