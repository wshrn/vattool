<template>
  <div class="app-shell">
    <TitleBar />
    <main ref="mainRef" class="main-stage">
      <div class="background-grid" aria-hidden="true"></div>
      <div class="glow glow-primary" aria-hidden="true"></div>
      <div class="glow glow-secondary" aria-hidden="true"></div>
      <section ref="cardRef" class="tech-card animate-slide-up">
        <header class="tech-card__header">
          <div class="tech-card__title">
            <span class="tech-card__badge">PY</span>
            <div>
              <p class="tech-card__eyebrow">Python Environment Pilot</p>
              <h1>Python 环境初始化助手</h1>
            </div>
          </div>
          <p class="tech-card__description">
            在当前目录快速检测并初始化 Python 运行环境，智能匹配常用国内镜像源，确保构建体验快速稳定。
          </p>
        </header>

        <div class="tech-divider" aria-hidden="true"></div>

        <div class="status-grid">
          <div
            v-for="item in statusLines"
            :key="item.label"
            class="status-card"
            :class="item.status.ok ? 'status-card--success' : 'status-card--error'"
          >
            <div class="status-card__header">
              <span class="status-card__icon" :class="item.status.ok ? 'ok' : 'error'">
                <component :is="item.status.ok ? CheckCircleIcon : XCircleIcon" class="w-4 h-4" />
              </span>
              <p class="status-card__label">{{ item.label }}</p>
            </div>
            <p class="status-card__message">{{ item.status.message }}</p>
            <p v-if="item.status.detail" class="status-card__detail">{{ item.status.detail }}</p>
          </div>
        </div>

        <div class="tech-divider" aria-hidden="true"></div>

        <div class="input-stack">
          <label class="tech-label">选择默认 pip 国内镜像源</label>
          <select v-model="selectedMirror" class="input-apple" :disabled="initializing">
            <option
              v-for="mirror in mirrorOptions"
              :key="mirror.value"
              :value="mirror.value"
            >
              {{ mirror.label }} · {{ mirror.value }}
            </option>
          </select>
          <p class="tech-helper">
            将写入全局 <code>pip.ini</code> 配置，并同步更新 trusted-host，避免证书校验问题。
          </p>
        </div>

        <div class="tech-divider" aria-hidden="true"></div>

        <div class="panel-footer">
          <div class="panel-meta">
            <p>当前 Python 路径：{{ status?.pythonPath ?? '未检测到' }}</p>
            <p>Scripts 目录：{{ status?.scriptsPath ?? '未检测到' }}</p>
            <p>当前镜像：{{ status?.pipMirrorConfigured.currentMirror ?? '未检测到' }}</p>
          </div>
          <button class="btn-primary" :disabled="initializing || loading" @click="handleInitialize">
            <span v-if="initializing" class="button-content">
              <svg class="w-4 h-4 animate-spin" viewBox="0 0 24 24" fill="none">
                <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
                <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8v4a4 4 0 00-4 4H4z" />
              </svg>
              <span>正在初始化...</span>
            </span>
            <span v-else class="button-content">
              <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none">
                <path
                  d="M4.75 12h14.5m0 0-4.5-4.5M19.25 12l-4.5 4.5"
                  stroke="currentColor"
                  stroke-width="1.5"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                />
              </svg>
              <span>开始初始化</span>
            </span>
          </button>
        </div>

        <transition-group name="list" tag="div" class="notification-stack">
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
import { CheckCircleIcon, XCircleIcon } from '@heroicons/vue/24/outline'
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
const mainRef = ref<HTMLElement | null>(null)
const cardRef = ref<HTMLElement | null>(null)
let notificationSeed = 0

const isTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
const appWindow = isTauri ? getCurrentWindow() : null
let resizeObserver: ResizeObserver | null = null

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
  if (isTauri) {
    updateWindowSize()
  }
  setTimeout(() => {
    const index = notifications.findIndex((tip) => tip.id === id)
    if (index >= 0) {
      notifications.splice(index, 1)
      if (isTauri) {
        updateWindowSize()
      }
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
    if (isTauri) {
      await updateWindowSize()
    }
  }
}

const parsePixels = (value: string | null) => (value ? Number.parseFloat(value) || 0 : 0)

const updateWindowSize = async () => {
  if (!isTauri || !appWindow || !cardRef.value) {
    return
  }

  await nextTick()

  const mainElement = mainRef.value
  const cardRect = cardRef.value.getBoundingClientRect()
  const mainStyle = mainElement ? window.getComputedStyle(mainElement) : null
  const paddingX = parsePixels(mainStyle?.paddingLeft ?? null) + parsePixels(mainStyle?.paddingRight ?? null)
  const paddingY = parsePixels(mainStyle?.paddingTop ?? null) + parsePixels(mainStyle?.paddingBottom ?? null)

  const titleBarElement = document.querySelector('.app-title-bar') as HTMLElement | null
  const titleBarHeight = titleBarElement?.getBoundingClientRect().height ?? 0

  const width = Math.ceil(cardRect.width + paddingX)
  const height = Math.ceil(cardRect.height + paddingY + titleBarHeight)

  const logicalSize = new LogicalSize(width, height)

  try {
    await appWindow.setSize(logicalSize)
    await appWindow.setMinSize(logicalSize)
    await appWindow.setMaxSize(logicalSize)
  } catch (error) {
    console.error('Failed to synchronize window size:', error)
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
    if (isTauri) {
      await updateWindowSize()
    }
  }
}

onMounted(async () => {
  await loadStatus()
  if (isTauri && typeof ResizeObserver !== 'undefined') {
    await updateWindowSize()
    resizeObserver = new ResizeObserver(() => {
      updateWindowSize()
    })
    const element = cardRef.value
    if (element) {
      resizeObserver.observe(element)
    }
  }
})

onBeforeUnmount(() => {
  resizeObserver?.disconnect()
})
</script>
