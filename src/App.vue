<template>
  <div ref="appShell" class="app-shell">
    <TitleBar />
    <main class="app-main">
      <section class="app-card animate-slide-up">
        <header class="app-card__header">
          <span class="card-badge">
            <span class="card-badge__dot" />
            <span>Environment Orchestrator</span>
          </span>
          <h1 class="app-card__title">Python 环境初始化助手</h1>
          <p class="app-card__subtitle">
            在当前目录快速检测并初始化 Python 运行环境，同时支持国内常见镜像源配置。
          </p>
        </header>

        <div class="status-grid">
          <div
            v-for="item in statusLines"
            :key="item.label"
            class="result-item"
            :class="item.status.ok ? 'success' : 'error'"
          >
            <div class="result-item__head">
              <span class="result-item__label">{{ item.label }}</span>
              <span class="result-item__chip" :class="item.status.ok ? 'chip--ok' : 'chip--error'">
                {{ item.status.ok ? '正常' : '异常' }}
              </span>
            </div>
            <p class="result-item__message">{{ item.status.message }}</p>
            <p v-if="item.status.detail" class="result-item__detail">{{ item.status.detail }}</p>
          </div>
        </div>

        <div class="input-stack">
          <label class="input-stack__label">选择默认 pip 国内镜像源</label>
          <div class="select-wrapper" :class="{ 'is-disabled': initializing }">
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
            <span class="select-glow" aria-hidden="true" />
          </div>
          <p class="input-stack__hint">
            将写入全局 <code>pip.ini</code> 配置，并自动加入对应的 trusted-host。
          </p>
        </div>

        <div class="card-footer">
          <div class="card-footer__meta">
            <p>当前 Python 路径：{{ status?.pythonPath ?? '未检测到' }}</p>
            <p>Scripts 目录：{{ status?.scriptsPath ?? '未检测到' }}</p>
            <p>当前镜像：{{ status?.pipMirrorConfigured.currentMirror ?? '未检测到' }}</p>
          </div>
          <button
            class="btn-primary"
            :disabled="initializing || loading"
            @click="handleInitialize"
          >
            <span v-if="initializing" class="btn-primary__loading">
              <svg class="w-4 h-4 animate-spin" viewBox="0 0 24 24" fill="none">
                <circle class="opacity-30" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
                <path class="opacity-80" fill="currentColor" d="M4 12a8 8 0 018-8v4a4 4 0 00-4 4H4z" />
              </svg>
              <span>正在初始化...</span>
            </span>
            <span v-else class="btn-primary__label">开始初始化</span>
          </button>
        </div>

        <transition-group name="list" tag="div" class="notification-tray">
          <div
            v-for="tip in notifications"
            :key="tip.id"
            class="notification"
            :class="tip.type"
          >
            <p class="notification__title">{{ tip.title }}</p>
            <p class="notification__message">{{ tip.message }}</p>
          </div>
        </transition-group>
      </section>
    </main>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, reactive, ref } from 'vue'
import { getCurrentWindow, LogicalSize } from '@tauri-apps/api/window'
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
const appShell = ref<HTMLElement | null>(null)
let notificationSeed = 0
let resizeObserver: ResizeObserver | null = null
let cleanupResizeListener: (() => void) | null = null

const appWindow = getCurrentWindow()

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

const ensureWindowSizeMatchesContent = async () => {
  if (!appShell.value || typeof window === 'undefined' || !('__TAURI_INTERNALS__' in window)) {
    return
  }

  const rect = appShell.value.getBoundingClientRect()
  if (!rect.width || !rect.height) {
    return
  }

  const size = new LogicalSize(Math.ceil(rect.width), Math.ceil(rect.height))

  try {
    await appWindow.setMinSize(size)
    await appWindow.setMaxSize(size)
    await appWindow.setSize(size)
  } catch (error) {
    console.warn('Failed to adjust window size:', error)
  }
}

const setupWindowAutoResize = () => {
  if (!appShell.value) {
    return
  }

  void ensureWindowSizeMatchesContent()

  if (typeof window === 'undefined') {
    return
  }

  if ('ResizeObserver' in window) {
    resizeObserver?.disconnect()
    resizeObserver = new ResizeObserver(() => {
      void ensureWindowSizeMatchesContent()
    })
    resizeObserver.observe(appShell.value)
  } else {
    const handler = () => {
      void ensureWindowSizeMatchesContent()
    }
    window.addEventListener('resize', handler)
    cleanupResizeListener = () => {
      window.removeEventListener('resize', handler)
    }
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
    void ensureWindowSizeMatchesContent()
  }
}

onMounted(async () => {
  await nextTick()
  setupWindowAutoResize()
  await loadStatus()
  await nextTick()
  void ensureWindowSizeMatchesContent()
})

onBeforeUnmount(() => {
  resizeObserver?.disconnect()
  resizeObserver = null
  if (cleanupResizeListener) {
    cleanupResizeListener()
    cleanupResizeListener = null
  }
  if (typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window) {
    void appWindow.setMinSize(null)
    void appWindow.setMaxSize(null)
  }
})
</script>
