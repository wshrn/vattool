<template>
  <div class="w-full h-screen overflow-hidden flex flex-col bg-[var(--app-surface-bg)] text-gray-900 dark:text-gray-100 transition-colors duration-300">
    <TitleBar>
      <template #actions>
        <button
          class="btn-ghost !px-3"
          :title="`当前主题：${themeLabel}`"
          @click="toggleTheme"
        >
          <component :is="isDark ? MoonIcon : SunIcon" class="w-4 h-4" />
        </button>
      </template>
    </TitleBar>

    <main class="flex-1 overflow-y-auto scrollbar-thin px-8 py-10">
      <div class="max-w-3xl mx-auto space-y-8">
        <section class="card-apple p-8 animate-fade-in">
          <header class="flex items-center justify-between">
            <div>
              <h1 class="text-2xl font-semibold text-gray-900 dark:text-gray-100">Python 环境初始化助手</h1>
              <p class="text-sm text-gray-500 dark:text-gray-400 mt-2">
                该工具用于检测当前目录下的 Python 可执行文件并自动完成环境变量与国内镜像源的配置。
              </p>
            </div>
            <div class="text-xs text-gray-500 dark:text-gray-400">
              <p>当前目录：{{ currentDirectory }}</p>
            </div>
          </header>

          <div class="mt-8 space-y-3">
            <div
              v-for="item in statusItems"
              :key="item.key"
              class="result-item flex items-center justify-between"
              :class="statusLevelClass(item.level)"
            >
              <div>
                <p class="font-medium">{{ item.label }}</p>
                <p v-if="item.detail" class="text-xs mt-1 text-gray-500 dark:text-gray-400">
                  {{ item.detail }}
                </p>
              </div>
              <span
                class="text-sm font-semibold"
                :class="item.ok ? 'text-green-600 dark:text-green-400' : 'text-red-500 dark:text-red-300'"
              >
                {{ item.ok ? '已完成' : '待处理' }}
              </span>
            </div>
          </div>
        </section>

        <section class="card-apple p-8 animate-slide-up">
          <div class="space-y-6">
            <div>
              <h2 class="text-xl font-semibold text-gray-900 dark:text-gray-100">国内源配置</h2>
              <p class="text-sm text-gray-500 dark:text-gray-400 mt-2">
                选择一个常见的 Python 国内源以提升 pip 下载速度，默认选择清华源。
              </p>
            </div>

            <div class="space-y-2">
              <label class="text-sm text-gray-600 dark:text-gray-300">镜像源</label>
              <select
                class="input-apple"
                v-model="selectedMirror"
              >
                <option
                  v-for="mirror in status?.mirrorSources ?? []"
                  :key="mirror.key"
                  :value="mirror.key"
                >
                  {{ mirror.label }} - {{ mirror.url }}
                </option>
              </select>
              <p class="text-xs text-gray-400" v-if="selectedMirrorInfo">
                {{ selectedMirrorInfo.description }}
              </p>
            </div>

            <div class="flex items-center justify-between">
              <div class="text-xs text-gray-400 dark:text-gray-500">
                <p v-if="lastMessage">{{ lastMessage }}</p>
              </div>
              <button
                class="btn-primary"
                :disabled="initializing"
                @click="handleInitialize"
              >
                <span v-if="initializing" class="flex items-center space-x-2">
                  <svg class="w-4 h-4 animate-spin" viewBox="0 0 24 24" fill="none" stroke="currentColor">
                    <circle class="opacity-25" cx="12" cy="12" r="10" stroke-width="4"></circle>
                    <path class="opacity-75" stroke-width="4" d="M4 12a8 8 0 018-8" stroke-linecap="round" />
                  </svg>
                  <span>初始化中...</span>
                </span>
                <span v-else>开始初始化</span>
              </button>
            </div>
          </div>
        </section>
      </div>
    </main>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { storeToRefs } from 'pinia'
import TitleBar from '@/components/TitleBar.vue'
import { useThemeStore } from '@/stores/theme'
import { MoonIcon, SunIcon } from '@heroicons/vue/24/outline'
import { PythonEnvAPI, SqlmapAPI, type PythonEnvStatusItem, type PythonEnvStatusResponse } from '@/api'

const themeStore = useThemeStore()
const { isDark, theme } = storeToRefs(themeStore)

const status = ref<PythonEnvStatusResponse | null>(null)
const initializing = ref(false)
const selectedMirror = ref('tsinghua')
const lastMessage = ref('')
const currentDirectory = ref('')
const statusItems = computed<PythonEnvStatusItem[]>(() => status.value?.items ?? [])

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

const statusLevelClass = (level: PythonEnvStatusItem['level']) => {
  switch (level) {
    case 'success':
      return 'success'
    case 'warning':
      return 'warning'
    case 'error':
      return 'error'
    default:
      return 'info'
  }
}

const selectedMirrorInfo = computed(() => {
  if (!status.value) return null
  return status.value.mirrorSources.find((m) => m.key === selectedMirror.value) ?? null
})

const toggleTheme = () => {
  const modes: Array<'light' | 'dark' | 'auto'> = ['light', 'dark', 'auto']
  const index = modes.indexOf(theme.value)
  const next = modes[(index + 1) % modes.length]
  void themeStore.setTheme(next)
}

const ensureOfflineLicense = async () => {
  if (typeof window === 'undefined' || !(window as any).__TAURI__) {
    return true
  }

  try {
    const result = await SqlmapAPI.validateOfflineKey()
    if (!result.isValid) {
      const reason = result.reason ?? '离线密钥无效'
      const expiresAt = result.expiresAt ? `，到期时间：${result.expiresAt}` : ''
      console.error(`离线密钥校验失败：${reason}${expiresAt}`)
      await window.__TAURI__.dialog?.message?.('认证失败', { title: '认证失败', type: 'error' })
      await window.__TAURI__.app?.exit?.(0)
      return false
    }
    return true
  } catch (error) {
    const detail = error instanceof Error ? error.message : String(error)
    console.error(`离线密钥校验失败：${detail}`)
    await window.__TAURI__.dialog?.message?.('认证失败', { title: '认证失败', type: 'error' })
    await window.__TAURI__.app?.exit?.(0)
    return false
  }
}

const refreshStatus = async () => {
  try {
    const response = await PythonEnvAPI.fetchStatus()
    status.value = response
    selectedMirror.value = response.selectedMirror
    currentDirectory.value = response.items.find((item) => item.key === 'python-executable')?.detail?.split('\n')[0] ?? ''
  } catch (error) {
    console.error('获取环境状态失败', error)
    lastMessage.value = '无法获取环境状态，请检查日志'
  }
}

const handleInitialize = async () => {
  initializing.value = true
  lastMessage.value = ''
  try {
    const result = await PythonEnvAPI.initializeEnvironment(selectedMirror.value)
    lastMessage.value = result.message
    if (result.status) {
      status.value = result.status
    }
  } catch (error) {
    const detail = error instanceof Error ? error.message : String(error)
    lastMessage.value = `初始化失败：${detail}`
  } finally {
    initializing.value = false
  }
}

onMounted(async () => {
  await themeStore.initialize()
  const licenseOk = await ensureOfflineLicense()
  if (!licenseOk) {
    return
  }
  await refreshStatus()
})
</script>
