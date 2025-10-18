<template>
  <div class="app-surface relative w-full overflow-hidden bg-slate-950 text-slate-100 transition-colors duration-500 ease-apple">
    <div class="absolute inset-0 pointer-events-none">
      <div class="absolute inset-0 bg-tech-grid opacity-40"></div>
      <div class="absolute -top-40 left-1/2 h-72 w-72 -translate-x-1/2 rounded-full bg-gradient-to-r from-sky-500/40 via-blue-500/20 to-transparent blur-3xl"></div>
      <div class="absolute bottom-[-160px] right-[-140px] h-80 w-80 rounded-full bg-gradient-to-br from-indigo-500/30 via-transparent to-emerald-400/25 blur-3xl"></div>
      <div class="absolute left-[-140px] top-1/3 h-64 w-64 rounded-full bg-gradient-to-tr from-purple-500/25 via-transparent to-sky-400/25 blur-3xl"></div>
    </div>

    <div class="relative z-10 flex w-full flex-col items-center">
      <TitleBar />
      <main ref="mainContainer" class="relative mx-auto w-fit px-10 pb-12 pt-8">
        <section ref="cardRef" class="tech-card relative w-[640px] overflow-hidden">
          <div class="pointer-events-none absolute inset-0 bg-card-lights"></div>
          <div class="relative z-10 space-y-8">
            <header class="space-y-6">
              <div class="flex items-center justify-between gap-4">
                <span class="inline-flex items-center gap-2 rounded-full bg-sky-500/10 px-4 py-1 text-[10px] font-semibold uppercase tracking-[0.35em] text-sky-200">
                  Python · INIT
                </span>
                <span class="hidden h-px flex-1 bg-gradient-to-r from-sky-500/40 via-transparent to-transparent md:block"></span>
              </div>
              <div class="space-y-2">
                <h1 class="text-3xl font-semibold text-white drop-shadow-sm">Python 环境初始化助手</h1>
                <p class="text-sm leading-relaxed text-slate-300">
                  快速检测并初始化当前目录的 Python 运行环境，自动校验常见变量、脚本路径及国内镜像源配置，让环境搭建一步到位。
                </p>
              </div>
            </header>

            <div v-if="loading" class="grid gap-3 md:grid-cols-2">
              <div v-for="index in 4" :key="index" class="status-tile skeleton"></div>
            </div>
            <div v-else-if="statusLines.length > 0" class="grid gap-3 md:grid-cols-2">
              <article
                v-for="item in statusLines"
                :key="item.label"
                class="status-tile"
                :class="item.status.ok ? 'status-ok' : 'status-error'"
              >
                <span class="status-icon" :class="item.status.ok ? 'status-icon-ok' : 'status-icon-error'">
                  <svg class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path
                      v-if="item.status.ok"
                      stroke-linecap="round"
                      stroke-linejoin="round"
                      d="M5 13l4 4L19 7"
                    />
                    <path
                      v-else
                      stroke-linecap="round"
                      stroke-linejoin="round"
                      d="M6 18L18 6M6 6l12 12"
                    />
                  </svg>
                </span>
                <div class="space-y-1">
                  <p class="text-sm font-medium text-slate-100">{{ item.label }}</p>
                  <p class="whitespace-pre-line text-xs text-slate-300">{{ item.status.message }}</p>
                  <p v-if="item.status.detail" class="text-xs text-slate-400">{{ item.status.detail }}</p>
                </div>
              </article>
            </div>
            <div v-else class="rounded-2xl border border-white/5 bg-white/5 px-4 py-6 text-center text-sm text-slate-300 backdrop-blur">
              暂未获取到环境状态，稍后将自动重试或手动重新初始化。
            </div>

            <div class="surface-panel space-y-3">
              <div class="flex items-center justify-between">
                <label class="text-sm font-medium text-slate-200">选择默认 pip 国内镜像源</label>
                <span class="text-[10px] uppercase tracking-[0.3em] text-sky-300">Mirror</span>
              </div>
              <div class="relative">
                <svg
                  class="pointer-events-none absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-sky-300"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="1.5"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  aria-hidden="true"
                >
                  <circle cx="12" cy="12" r="9" />
                  <path d="M2.05 11h19.9M12 2.05a14.2 14.2 0 010 19.9M12 2.05a14.2 14.2 0 000 19.9" />
                </svg>
                <select
                  v-model="selectedMirror"
                  class="input-apple select-tech pl-10"
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
              </div>
              <p class="text-xs text-slate-400">
                将写入全局 <code class="rounded bg-white/10 px-1 py-0.5 text-[11px] text-slate-200">pip.ini</code> 配置，并自动添加对应的 trusted-host，保障国内网络环境下的稳定下载。
              </p>
            </div>

            <div class="flex flex-col gap-6 md:flex-row md:items-center md:justify-between">
              <div class="tech-meta space-y-1 text-xs text-slate-400">
                <p><span class="meta-label">Python</span>{{ status?.pythonPath ?? '未检测到' }}</p>
                <p><span class="meta-label">Scripts</span>{{ status?.scriptsPath ?? '未检测到' }}</p>
                <p><span class="meta-label">Mirror</span>{{ status?.pipMirrorConfigured.currentMirror ?? '未检测到' }}</p>
              </div>
              <button
                class="btn-primary btn-primary-glow px-8 py-3 text-sm"
                :disabled="initializing || loading"
                @click="handleInitialize"
              >
                <span v-if="initializing" class="flex items-center gap-2">
                  <svg class="h-4 w-4 animate-spin" viewBox="0 0 24 24" fill="none">
                    <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
                    <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8v4a4 4 0 00-4 4H4z" />
                  </svg>
                  <span>正在初始化...</span>
                </span>
                <span v-else-if="loading" class="flex items-center gap-2">
                  <svg class="h-4 w-4 animate-spin" viewBox="0 0 24 24" fill="none">
                    <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
                    <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8v4a4 4 0 00-4 4H4z" />
                  </svg>
                  <span>加载状态...</span>
                </span>
                <span v-else class="flex items-center gap-2">
                  <svg class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8">
                    <path stroke-linecap="round" stroke-linejoin="round" d="M12 5v14m7-7H5" />
                  </svg>
                  <span>开始初始化</span>
                </span>
              </button>
            </div>

            <transition-group name="list" tag="div" class="space-y-2">
              <div
                v-for="tip in notifications"
                :key="tip.id"
                class="notification-tile"
                :class="`notification-${tip.type}`"
              >
                <div class="flex items-start justify-between gap-4">
                  <div>
                    <p class="text-sm font-medium text-slate-100">{{ tip.title }}</p>
                    <p class="mt-1 text-xs text-slate-300">{{ tip.message }}</p>
                  </div>
                  <span class="inline-flex h-6 w-6 items-center justify-center rounded-full bg-white/10 text-[10px] tracking-[0.2em] text-slate-300">
                    log
                  </span>
                </div>
              </div>
            </transition-group>
          </div>
        </section>
      </main>
    </div>
  </div>
</template>

<script setup lang="ts">
import { getCurrentWindow, LogicalSize } from '@tauri-apps/api/window'
import { computed, nextTick, onBeforeUnmount, onMounted, reactive, ref } from 'vue'
import TitleBar from './components/TitleBar.vue'
import { fetchPythonEnvStatus, initializePythonEnvironment, type PythonEnvStatus } from './services/pythonEnv'

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
const mainContainer = ref<HTMLElement | null>(null)
const cardRef = ref<HTMLElement | null>(null)
let notificationSeed = 0
let resizeObserver: ResizeObserver | null = null
let resizeFrame = 0
let lastAppliedSize: { width: number; height: number } | null = null

const TITLE_BAR_HEIGHT = 36
const isTauri = typeof window !== 'undefined' && '__TAURI_IPC__' in window
const currentWindow = isTauri ? getCurrentWindow() : null

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
  void nextTick().then(scheduleResize)
  setTimeout(() => {
    const index = notifications.findIndex((tip) => tip.id === id)
    if (index >= 0) {
      notifications.splice(index, 1)
      void nextTick().then(scheduleResize)
    }
  }, 5000)
}

const adjustWindowSize = async () => {
  if (!isTauri || !cardRef.value || !mainContainer.value) {
    return
  }
  const cardRect = cardRef.value.getBoundingClientRect()
  const containerStyles = window.getComputedStyle(mainContainer.value)
  const paddingX = parseFloat(containerStyles.paddingLeft) + parseFloat(containerStyles.paddingRight)
  const paddingY = parseFloat(containerStyles.paddingTop) + parseFloat(containerStyles.paddingBottom)
  const width = Math.ceil(cardRect.width + paddingX)
  const height = Math.ceil(cardRect.height + paddingY + TITLE_BAR_HEIGHT)

  if (lastAppliedSize && lastAppliedSize.width === width && lastAppliedSize.height === height) {
    return
  }

  lastAppliedSize = { width, height }

  try {
    if (!currentWindow) {
      return
    }
    const logicalSize = new LogicalSize(width, height)
    await currentWindow.setMinSize(logicalSize)
    await currentWindow.setSize(logicalSize)
  } catch (error) {
    console.error('调整窗口大小失败:', error)
  }
}

const scheduleResize = () => {
  if (!isTauri) {
    return
  }
  if (resizeFrame) {
    cancelAnimationFrame(resizeFrame)
  }
  resizeFrame = requestAnimationFrame(() => {
    resizeFrame = 0
    void adjustWindowSize()
  })
}

const setupResizeObserver = () => {
  if (!cardRef.value || typeof ResizeObserver === 'undefined') {
    scheduleResize()
    return
  }
  resizeObserver = new ResizeObserver(() => {
    scheduleResize()
  })
  resizeObserver.observe(cardRef.value)
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
  } catch (error) {
    console.error(error)
    pushNotification({
      title: '状态读取失败',
      message: error instanceof Error ? error.message : String(error),
      type: 'error',
    })
  } finally {
    loading.value = false
    await nextTick()
    scheduleResize()
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
  } catch (error) {
    console.error(error)
    pushNotification({
      title: '初始化失败',
      message: error instanceof Error ? error.message : String(error),
      type: 'error',
    })
  } finally {
    initializing.value = false
    await nextTick()
    scheduleResize()
  }
}

onMounted(async () => {
  setupResizeObserver()
  if (isTauri && currentWindow) {
    try {
      await currentWindow.setResizable(false)
    } catch (error) {
      console.error('无法设置窗口不可调整大小:', error)
    }
  }
  scheduleResize()
  await loadStatus()
})

onBeforeUnmount(() => {
  if (resizeObserver) {
    resizeObserver.disconnect()
    resizeObserver = null
  }
  if (resizeFrame) {
    cancelAnimationFrame(resizeFrame)
    resizeFrame = 0
  }
})
</script>
