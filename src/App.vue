<template>
  <div class="app-shell text-slate-900 dark:text-slate-100">
    <TitleBar />
    <main class="app-main">
      <section ref="cardRef" class="card-apple space-y-8">
        <div class="card-accent" aria-hidden="true"></div>
        <header class="card-header">
          <p class="card-eyebrow">Python Environment Toolkit</p>
          <h1 class="card-title">Python 环境初始化助手</h1>
          <p class="card-subtitle">在当前目录快速检测并初始化 Python 运行环境，同时支持国内常见镜像源配置。</p>
        </header>

        <div class="status-grid">
          <div
            v-for="item in statusLines"
            :key="item.label"
            class="result-item"
            :class="item.status.ok ? 'success' : 'error'"
          >
            <div class="result-item__header">
              <span class="status-dot" :class="item.status.ok ? 'status-dot--success' : 'status-dot--error'"></span>
              <p class="result-item__title">{{ item.label }}</p>
            </div>
            <p class="result-item__message">{{ item.status.message }}</p>
            <p v-if="item.status.detail" class="result-item__detail">{{ item.status.detail }}</p>
          </div>
        </div>

        <div class="input-group">
          <label class="input-label">选择默认 pip 国内镜像源</label>
          <select v-model="selectedMirror" class="input-apple" :disabled="initializing">
            <option
              v-for="mirror in mirrorOptions"
              :key="mirror.value"
              :value="mirror.value"
            >
              {{ mirror.label }} - {{ mirror.value }}
            </option>
          </select>
          <p class="input-helper">
            将写入全局 <code>pip.ini</code> 配置，并自动加入对应的 trusted-host。
          </p>
        </div>

        <div class="card-footer">
          <div class="card-meta">
            <p>当前 Python 路径：{{ status?.pythonPath ?? '未检测到' }}</p>
            <p>Scripts 目录：{{ status?.scriptsPath ?? '未检测到' }}</p>
            <p>当前镜像：{{ status?.pipMirrorConfigured.currentMirror ?? '未检测到' }}</p>
          </div>
          <button class="btn-primary" :disabled="initializing || loading" @click="handleInitialize">
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
import { computed, nextTick, onMounted, reactive, ref, watch } from 'vue'
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
let notificationSeed = 0
const cardRef = ref<HTMLElement | null>(null)
const isTauriEnvironment = typeof window !== 'undefined' && '__TAURI__' in window
const appWindow = isTauriEnvironment ? getCurrentWindow() : null

const adjustWindowSize = async () => {
  await nextTick()
  if (!appWindow || !isTauriEnvironment) {
    return
  }
  const card = cardRef.value
  if (!card) {
    return
  }
  const rect = card.getBoundingClientRect()
  const horizontalPadding = 64
  const verticalPadding = 96
  const width = Math.max(Math.ceil(rect.width + horizontalPadding), 560)
  const height = Math.max(Math.ceil(rect.height + verticalPadding), 520)
  try {
    await appWindow.setSize(new LogicalSize(width, height))
  } catch (error) {
    console.error('Failed to resize window:', error)
  }
}

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
      adjustWindowSize()
    }
  }, 5000)
  adjustWindowSize()
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
    await adjustWindowSize()
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
  } catch (error) {
    console.error(error)
    pushNotification({
      title: '初始化失败',
      message: error instanceof Error ? error.message : String(error),
      type: 'error',
    })
  } finally {
    initializing.value = false
    await adjustWindowSize()
  }
}

onMounted(async () => {
  await loadStatus()
  await adjustWindowSize()
})

watch(
  () => [status.value, notifications.length, initializing.value, selectedMirror.value],
  () => {
    adjustWindowSize()
  },
  { flush: 'post' }
)
</script>
