<template>
  <div class="min-h-screen flex flex-col bg-[var(--app-surface-bg)] text-gray-900 dark:text-gray-100 transition-colors duration-300">
    <TitleBar />

    <main class="flex-1 overflow-y-auto scrollbar-thin">
      <div class="max-w-4xl mx-auto px-6 py-10">
        <header class="flex items-center justify-between mb-8">
          <div>
            <h1 class="text-3xl font-semibold mb-1">Python 3 环境智能助手</h1>
            <p class="text-sm text-gray-500 dark:text-gray-400">自动检测并初始化当前目录下的 Python 环境。</p>
          </div>
          <button
            class="p-3 rounded-2xl bg-gray-100 dark:bg-gray-800 text-gray-600 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-gray-700 transition-colors"
            :title="`当前主题：${themeLabel}`"
            @click="toggleTheme"
          >
            <component :is="isDark ? MoonIcon : SunIcon" class="w-5 h-5" />
          </button>
        </header>

        <section class="card-apple p-6 space-y-6">
          <div class="space-y-3">
            <StatusLine
              icon="cpu"
              title="Python 可执行文件"
              :status="statuses.pythonExecutable"
            />
            <StatusLine
              icon="variable"
              title="PYTHON3 变量配置"
              :status="statuses.pythonRootVariable"
            />
            <StatusLine
              icon="path"
              title="PATH 中的 Python 路径"
              :status="statuses.pythonPath"
            />
            <StatusLine
              icon="script"
              title="PATH 中的 Scripts 目录"
              :status="statuses.pythonScripts"
            />
            <StatusLine
              icon="globe"
              title="PIP 国内源设置"
              :status="statuses.pythonMirror"
            />
          </div>

          <div class="bg-gray-50 dark:bg-gray-800/50 rounded-2xl p-4 border border-gray-200 dark:border-gray-700">
            <label for="mirror" class="block text-sm font-medium text-gray-600 dark:text-gray-300 mb-2">选择 Python 包国内镜像源</label>
            <Listbox v-model="selectedMirror">
              <div class="relative mt-1">
                <ListboxButton class="relative w-full cursor-pointer rounded-2xl bg-white dark:bg-gray-900 py-3 pl-4 pr-10 text-left border border-gray-200 dark:border-gray-700 focus:outline-none focus:ring-2 focus:ring-apple-blue focus:border-transparent">
                  <span class="block truncate">{{ selectedMirror.label }}</span>
                  <span class="pointer-events-none absolute inset-y-0 right-0 flex items-center pr-3 text-gray-500">
                    <ChevronUpDownIcon class="h-5 w-5" aria-hidden="true" />
                  </span>
                </ListboxButton>
                <transition leave-active-class="transition duration-100 ease-in" leave-from-class="opacity-100" leave-to-class="opacity-0">
                  <ListboxOptions class="absolute z-10 mt-2 max-h-60 w-full overflow-auto rounded-2xl bg-white dark:bg-gray-900 py-1 text-base shadow-lg ring-1 ring-black/5 focus:outline-none sm:text-sm">
                    <ListboxOption
                      v-for="mirror in mirrors"
                      :key="mirror.id"
                      :value="mirror"
                      v-slot="{ active, selected }"
                    >
                      <li :class="[
                        active ? 'bg-apple-blue/10 text-apple-blue dark:bg-apple-blue/20' : 'text-gray-900 dark:text-gray-100',
                        'relative cursor-pointer select-none py-2 pl-10 pr-4'
                      ]">
                        <span :class="[selected ? 'font-medium' : 'font-normal', 'block truncate']">{{ mirror.label }}</span>
                        <span
                          v-if="selected"
                          class="absolute inset-y-0 left-0 flex items-center pl-3 text-apple-blue"
                        >
                          <CheckIcon class="h-5 w-5" aria-hidden="true" />
                        </span>
                      </li>
                    </ListboxOption>
                  </ListboxOptions>
                </transition>
              </div>
            </Listbox>
            <p class="mt-2 text-xs text-gray-500 dark:text-gray-400">
              默认选择清华大学镜像，如果需要，可以切换到阿里云、中科大或华为云等镜像。
            </p>
          </div>

          <div class="flex items-center justify-between flex-wrap gap-3">
            <div class="text-xs text-gray-500 dark:text-gray-400">
              离线密钥状态：<span :class="licenseStatusClass">{{ licenseMessage }}</span>
            </div>
            <button
              class="btn-primary flex items-center space-x-2"
              :disabled="isInitializing"
              @click="initialize"
            >
              <svg
                v-if="isInitializing"
                class="w-5 h-5 text-white loading-spinner"
                viewBox="0 0 24 24"
                fill="none"
                xmlns="http://www.w3.org/2000/svg"
              >
                <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
                <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8v4a4 4 0 00-4 4H4z" />
              </svg>
              <span>{{ isInitializing ? '正在初始化...' : '开始初始化' }}</span>
            </button>
          </div>
        </section>
      </div>
    </main>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref } from 'vue'
import { storeToRefs } from 'pinia'
import { invoke } from '@tauri-apps/api/tauri'
import { MoonIcon, SunIcon, CheckIcon, ChevronUpDownIcon } from '@heroicons/vue/24/outline'
import { Listbox, ListboxButton, ListboxOption, ListboxOptions } from '@headlessui/vue'
import TitleBar from './components/TitleBar.vue'
import StatusLine from './components/StatusLine.vue'
import { useThemeStore } from './stores/theme'

interface StatusInfo {
  ok: boolean
  message: string
  level: 'success' | 'warning' | 'error' | 'info'
}

interface PythonStatusReport {
  pythonExecutable: StatusInfo
  pythonRootVariable: StatusInfo
  pythonPath: StatusInfo
  pythonScripts: StatusInfo
  pythonMirror: StatusInfo
}

interface MirrorOption {
  id: string
  label: string
  indexUrl: string
  trustedHost: string
}

const mirrors: MirrorOption[] = [
  {
    id: 'tsinghua',
    label: '清华大学镜像 https://pypi.tuna.tsinghua.edu.cn/simple',
    indexUrl: 'https://pypi.tuna.tsinghua.edu.cn/simple',
    trustedHost: 'pypi.tuna.tsinghua.edu.cn',
  },
  {
    id: 'aliyun',
    label: '阿里云镜像 https://mirrors.aliyun.com/pypi/simple',
    indexUrl: 'https://mirrors.aliyun.com/pypi/simple',
    trustedHost: 'mirrors.aliyun.com',
  },
  {
    id: 'ustc',
    label: '中国科技大学 https://mirrors.ustc.edu.cn/pypi/web/simple',
    indexUrl: 'https://mirrors.ustc.edu.cn/pypi/web/simple',
    trustedHost: 'mirrors.ustc.edu.cn',
  },
  {
    id: 'huawei',
    label: '华为云镜像 https://repo.huaweicloud.com/repository/pypi/simple',
    indexUrl: 'https://repo.huaweicloud.com/repository/pypi/simple',
    trustedHost: 'repo.huaweicloud.com',
  },
]

const selectedMirror = ref<MirrorOption>(mirrors[0])
const statuses = reactive<PythonStatusReport>({
  pythonExecutable: { ok: false, message: '正在检测...', level: 'info' },
  pythonRootVariable: { ok: false, message: '正在检测...', level: 'info' },
  pythonPath: { ok: false, message: '正在检测...', level: 'info' },
  pythonScripts: { ok: false, message: '正在检测...', level: 'info' },
  pythonMirror: { ok: false, message: '正在检测...', level: 'info' },
})

const isInitializing = ref(false)
const licenseStatus = ref<'unknown' | 'valid' | 'invalid'>('unknown')
const licenseMessage = ref('正在校验离线密钥...')

const themeStore = useThemeStore()
const { isDark, theme } = storeToRefs(themeStore)
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

const toggleTheme = () => {
  const modes: Array<'light' | 'dark' | 'auto'> = ['light', 'dark', 'auto']
  const currentIndex = modes.indexOf(theme.value)
  const next = modes[(currentIndex + 1) % modes.length]
  void themeStore.setTheme(next)
}

const licenseStatusClass = computed(() => {
  switch (licenseStatus.value) {
    case 'valid':
      return 'text-apple-green font-medium'
    case 'invalid':
      return 'text-apple-red font-medium'
    default:
      return 'text-gray-500 dark:text-gray-400'
  }
})

const refreshStatus = async () => {
  try {
    const report = await invoke<PythonStatusReport>('fetch_python_status', {
      preferredMirror: selectedMirror.value.indexUrl,
    })
    Object.assign(statuses, report)
  } catch (error) {
    console.error('获取 Python 状态失败', error)
  }
}

const validateOfflineKey = async () => {
  try {
    const result = await invoke<{ isValid: boolean; reason?: string; expiresAt?: string }>('validate_offline_key')
    if (!result.isValid) {
      licenseStatus.value = 'invalid'
      licenseMessage.value = result.reason ? `失败：${result.reason}` : '离线密钥无效'
    } else {
      licenseStatus.value = 'valid'
      licenseMessage.value = result.expiresAt ? `有效，过期时间：${result.expiresAt}` : '密钥校验成功'
    }
  } catch (error) {
    licenseStatus.value = 'invalid'
    licenseMessage.value = error instanceof Error ? error.message : String(error)
  }
}

const initialize = async () => {
  isInitializing.value = true
  try {
    await invoke('initialize_python_environment', {
      preferredMirror: selectedMirror.value,
    })
    await refreshStatus()
  } catch (error) {
    console.error('初始化失败', error)
  } finally {
    isInitializing.value = false
  }
}

onMounted(async () => {
  await validateOfflineKey()
  await refreshStatus()
})
</script>
