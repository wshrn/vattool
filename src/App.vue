<template>
  <div ref="appShellRef" class="app-shell">
    <TitleBar ref="titleBarRef" />
    <main ref="contentAreaRef" class="relative flex w-full justify-center px-6 py-6">
      <div aria-hidden="true" class="pointer-events-none absolute inset-0">
        <div class="absolute -top-24 right-0 h-64 w-64 rounded-full bg-gradient-to-br from-sky-400/40 via-blue-500/30 to-purple-500/40 blur-3xl"></div>
        <div class="absolute bottom-[-80px] left-[-120px] h-72 w-72 rounded-full bg-gradient-to-br from-emerald-400/30 via-cyan-400/20 to-transparent blur-3xl"></div>
        <div class="absolute top-1/2 left-1/2 h-48 w-48 -translate-x-1/2 -translate-y-1/2 rounded-full bg-gradient-to-br from-white/40 via-blue-100/20 to-transparent blur-2xl dark:from-slate-500/30 dark:via-blue-500/20"></div>
      </div>
      <section ref="cardRef" class="card-apple w-full max-w-3xl space-y-8">
        <header class="space-y-2">
          <span class="inline-flex items-center rounded-full bg-blue-500/10 px-3 py-1 text-[11px] font-medium uppercase tracking-[0.3em] text-blue-600 dark:text-blue-300">
            Smart Environment Toolkit
          </span>
          <h1 class="text-3xl font-semibold tracking-tight">Python 环境初始化助手</h1>
          <p class="text-sm leading-relaxed text-gray-600 dark:text-slate-300">
            在当前目录快速检测并初始化 Python 运行环境，同时支持国内常见镜像源配置。
          </p>
        </header>

        <div class="grid gap-3">
          <div
            v-for="item in statusLines"
            :key="item.label"
            class="result-item"
            :class="item.status.ok ? 'success' : 'error'"
          >
            <p class="text-sm font-medium">{{ item.label }}</p>
            <p class="text-xs text-gray-600 dark:text-gray-400 mt-1 whitespace-pre-line">
              {{ item.status.message }}
            </p>
            <p v-if="item.status.detail" class="text-xs text-gray-500 dark:text-gray-400 mt-1">{{ item.status.detail }}</p>
          </div>
        </div>

        <div class="space-y-3">
          <label class="block text-sm font-medium text-gray-800 dark:text-gray-200">选择默认 pip 国内镜像源</label>
          <select
            v-model="selectedMirror"
            class="input-apple"
            :disabled="initializing"
          >
            <option
              v-for="mirror in mirrorOptions"
              :key="mirror.value"
              :value="mirror.value"
            >
              {{ mirror.label }} - {{ mirror.value }}
            </option>
          </select>
          <p class="text-xs text-gray-500 dark:text-gray-400">
            将写入全局 <code>pip.ini</code> 配置，并自动加入对应的 trusted-host。
          </p>
        </div>

        <div class="flex flex-col gap-4 rounded-2xl border border-white/10 bg-white/30 p-4 text-xs text-gray-600 shadow-inner backdrop-blur-lg dark:border-white/5 dark:bg-slate-900/50 dark:text-slate-300 sm:flex-row sm:items-center sm:justify-between">
          <div class="space-y-1">
            <p>当前 Python 路径：{{ status?.pythonPath ?? '未检测到' }}</p>
            <p>Scripts 目录：{{ status?.scriptsPath ?? '未检测到' }}</p>
            <p>当前镜像：{{ status?.pipMirrorConfigured.currentMirror ?? '未检测到' }}</p>
          </div>
          <button
            class="btn-primary px-6 py-3 text-sm shadow-lg shadow-blue-500/20"
            :disabled="initializing || loading"
            @click="handleInitialize"
          >
            <span v-if="initializing" class="flex items-center space-x-2">
              <svg class="w-4 h-4 animate-spin" viewBox="0 0 24 24" fill="none">
                <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
                <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8v4a4 4 0 00-4 4H4z" />
              </svg>
              <span>正在初始化...</span>
            </span>
            <span v-else>开始初始化</span>
          </button>
        </div>

        <transition-group name="list" tag="div" class="space-y-2">
          <div
            v-for="tip in notifications"
            :key="tip.id"
            class="notification"
            :class="tip.type"
          >
            <p class="text-sm font-medium">{{ tip.title }}</p>
            <p class="text-xs mt-1">{{ tip.message }}</p>
          </div>
        </transition-group>
      </section>
    </main>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, reactive, ref, watch } from 'vue'
import TitleBar from './components/TitleBar.vue'
import { fetchPythonEnvStatus, initializePythonEnvironment, type PythonEnvStatus } from './services/pythonEnv'
import { getCurrentWindow, LogicalSize } from '@tauri-apps/api/window'

interface NotificationItem {
  id: number
  title: string
  message: string
  type: 'success' | 'error' | 'info' | 'warning'
}

const status = ref<PythonEnvStatus | null>(null)
const loading = ref(true)
const initializing = ref(false)
const selectedMirror = ref('https://pypi.tuna.tsinghua.edu.cn/simple')
const notifications = reactive<NotificationItem[]>([])
let notificationSeed = 0
const appShellRef = ref<HTMLElement | null>(null)
const contentAreaRef = ref<HTMLElement | null>(null)
const cardRef = ref<HTMLElement | null>(null)
const titleBarRef = ref<{ rootEl: HTMLElement | null } | null>(null)
const appWindow = getCurrentWindow()
let resizeObserver: ResizeObserver | null = null
let resizeAnimationFrame: number | null = null

const mirrorOptions = computed(() => status.value?.mirrorCandidates ?? [
  {
    label: '清华大学 TUNA',
    value: 'https://pypi.tuna.tsinghua.edu.cn/simple',
    trustedHosts: ['pypi.tuna.tsinghua.edu.cn'],
  },
])

const statusLines = computed(() => {
  if (!status.value) {
    return []
  }
  return [
    { label: '当前目录检测', status: status.value.pythonPresent },
    { label: 'python3 环境变量', status: status.value.pythonEnvVar },
    { label: 'PATH 中的 Python', status: status.value.pathConfigured },
    { label: 'PATH 中的 Scripts', status: status.value.scriptsConfigured },
    { label: 'Pip 镜像配置', status: status.value.pipMirrorConfigured },
  ]
})

const pushNotification = (item: Omit<NotificationItem, 'id'>) => {
  const id = ++notificationSeed
  notifications.push({ id, ...item })
  setTimeout(() => {
    const index = notifications.findIndex((tip) => tip.id === id)
    if (index >= 0) {
      notifications.splice(index, 1)
    }
  }, 5000)
}

const synchronizeMirrorSelection = () => {
  const current = status.value?.pipMirrorConfigured.currentMirror
  if (current && mirrorOptions.value.some((item) => item.value === current)) {
    selectedMirror.value = current
  } else if (mirrorOptions.value.length > 0) {
    selectedMirror.value = mirrorOptions.value[0].value
  }
}

const loadStatus = async () => {
  loading.value = true
  try {
    status.value = await fetchPythonEnvStatus()
    synchronizeMirrorSelection()
    await nextTick()
    synchronizeWindowSize()
  } catch (error) {
    console.error(error)
    pushNotification({
      title: '状态读取失败',
      message: error instanceof Error ? error.message : String(error),
      type: 'error',
    })
  } finally {
    loading.value = false
  }
}

const handleInitialize = async () => {
  if (!status.value) {
    return
  }
  initializing.value = true
  try {
    status.value = await initializePythonEnvironment({ mirror: selectedMirror.value })
    synchronizeMirrorSelection()
    pushNotification({
      title: '初始化完成',
      message: 'Python 环境变量与国内镜像已配置。',
      type: 'success',
    })
    await nextTick()
    synchronizeWindowSize()
  } catch (error) {
    console.error(error)
    pushNotification({
      title: '初始化失败',
      message: error instanceof Error ? error.message : String(error),
      type: 'error',
    })
  } finally {
    initializing.value = false
  }
}

onMounted(async () => {
  if (appShellRef.value) {
    resizeObserver = new ResizeObserver(() => {
      synchronizeWindowSize()
    })
    resizeObserver.observe(appShellRef.value)
  }
  await nextTick()
  synchronizeWindowSize()
  await loadStatus()
})

onBeforeUnmount(async () => {
  resizeObserver?.disconnect()
  if (resizeAnimationFrame !== null) {
    cancelAnimationFrame(resizeAnimationFrame)
  }
  try {
    await appWindow.setMinSize(null)
    await appWindow.setMaxSize(null)
  } catch (error) {
    console.error('Failed to reset window constraints:', error)
  }
})

const synchronizeWindowSize = () => {
  if (!appShellRef.value) {
    return
  }

  if (resizeAnimationFrame !== null) {
    cancelAnimationFrame(resizeAnimationFrame)
  }

  const element = cardRef.value ?? appShellRef.value

  resizeAnimationFrame = requestAnimationFrame(async () => {
    const cardRect = element.getBoundingClientRect()
    const titleBarElement = titleBarRef.value?.rootEl ?? null
    const titleBarRect = titleBarElement?.getBoundingClientRect()

    let paddingX = 0
    let paddingY = 0

    if (contentAreaRef.value) {
      const styles = window.getComputedStyle(contentAreaRef.value)
      const paddingLeft = Number.parseFloat(styles.paddingLeft) || 0
      const paddingRight = Number.parseFloat(styles.paddingRight) || 0
      const paddingTop = Number.parseFloat(styles.paddingTop) || 0
      const paddingBottom = Number.parseFloat(styles.paddingBottom) || 0
      paddingX = paddingLeft + paddingRight
      paddingY = paddingTop + paddingBottom
    }

    const titleBarHeight = titleBarRect?.height ?? 0
    const titleBarWidth = titleBarElement?.scrollWidth ?? titleBarRect?.width ?? 0

    const contentWidth = cardRect.width + paddingX
    const contentHeight = cardRect.height + paddingY + titleBarHeight
    const verticalBuffer = 16

    const width = Math.ceil(Math.max(contentWidth, titleBarWidth))
    const height = Math.ceil(contentHeight + verticalBuffer)

    try {
      const size = new LogicalSize(width, height)
      await appWindow.setSize(size)
      await appWindow.setMinSize(size)
      await appWindow.setMaxSize(size)
    } catch (error) {
      console.error('Failed to synchronize window size:', error)
    }
  })
}

watch(status, async () => {
  await nextTick()
  synchronizeWindowSize()
})

watch(
  () => notifications.length,
  async () => {
    await nextTick()
    synchronizeWindowSize()
  },
)

watch(initializing, async () => {
  await nextTick()
  synchronizeWindowSize()
})
</script>
