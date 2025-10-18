<template>
  <div
    class="relative min-h-screen overflow-hidden bg-gradient-to-br from-[#eef2ff] via-[#f5f7ff] to-[#e2e8f0] text-slate-900 transition-colors duration-500 ease-apple dark:from-[#020617] dark:via-[#0f172a] dark:to-[#020617] dark:text-slate-100"
  >
    <div class="pointer-events-none absolute inset-0 overflow-hidden" aria-hidden="true">
      <div class="tech-grid"></div>
      <div class="tech-glow top-[-18%] left-[-12%] md:left-[-6%]"></div>
      <div class="tech-glow-secondary bottom-[-28%] right-[-18%]"></div>
      <div class="tech-orbit"></div>
    </div>

    <TitleBar />

    <main class="relative z-10 mx-auto max-w-5xl px-6 py-12 md:py-16">
      <section class="card-apple relative overflow-hidden space-y-9 p-10 animate-slide-up">
        <div class="card-apple__background" aria-hidden="true"></div>
        <div class="card-apple__orb card-apple__orb--one" aria-hidden="true"></div>
        <div class="card-apple__orb card-apple__orb--two" aria-hidden="true"></div>

        <header class="relative z-10 space-y-4">
          <span class="neon-chip">智能初始化</span>
          <h1 class="text-3xl font-semibold tracking-tight md:text-4xl">Python 环境初始化助手</h1>
          <p class="text-sm text-slate-600 dark:text-slate-300 md:text-base">
            在当前目录快速检测并初始化 Python 运行环境，提供实时反馈与动态可视化，并内置国内常用镜像源加速配置。
          </p>
        </header>

        <div class="relative z-10 grid gap-3">
          <div
            v-for="item in statusLines"
            :key="item.label"
            class="result-item"
            :class="item.status.ok ? 'success' : 'error'"
          >
            <div class="flex items-start justify-between gap-3">
              <p class="text-sm font-medium">{{ item.label }}</p>
              <span class="status-indicator" :class="item.status.ok ? 'ok' : 'fail'">
                <svg
                  v-if="item.status.ok"
                  class="h-4 w-4"
                  viewBox="0 0 20 20"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2"
                >
                  <path stroke-linecap="round" stroke-linejoin="round" d="M5 10l3 3 7-7" />
                </svg>
                <svg
                  v-else
                  class="h-4 w-4"
                  viewBox="0 0 20 20"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2"
                >
                  <path stroke-linecap="round" stroke-linejoin="round" d="M6 6l8 8M14 6l-8 8" />
                </svg>
              </span>
            </div>
            <p class="mt-2 text-xs text-slate-600 dark:text-slate-300 whitespace-pre-line">
              {{ item.status.message }}
            </p>
            <p v-if="item.status.detail" class="mt-1 text-xs text-slate-500 dark:text-slate-400">{{ item.status.detail }}</p>
          </div>
        </div>

        <div class="relative z-10 space-y-3">
          <label class="block text-sm font-medium text-slate-700 dark:text-slate-200">选择默认 pip 国内镜像源</label>
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
          <p class="text-xs text-slate-500 dark:text-slate-400">
            将写入全局 <code>pip.ini</code> 配置，并自动加入对应的 trusted-host。
          </p>
        </div>

        <div class="relative z-10 flex flex-col gap-4 md:flex-row md:items-center md:justify-between">
          <div class="space-y-1 text-xs text-slate-500 dark:text-slate-400">
            <p>当前 Python 路径：{{ status?.pythonPath ?? '未检测到' }}</p>
            <p>Scripts 目录：{{ status?.scriptsPath ?? '未检测到' }}</p>
            <p>当前镜像：{{ status?.pipMirrorConfigured.currentMirror ?? '未检测到' }}</p>
          </div>
          <button
            class="btn-primary px-7 py-3 text-sm"
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
            <span v-else class="flex items-center gap-2">
              <span>开始初始化</span>
              <svg class="h-4 w-4" viewBox="0 0 20 20" fill="none" stroke="currentColor" stroke-width="2">
                <path stroke-linecap="round" stroke-linejoin="round" d="M5 10h8M11 6l4 4-4 4" />
              </svg>
            </span>
          </button>
        </div>

        <transition-group name="list" tag="div" class="relative z-10 space-y-2">
          <div
            v-for="tip in notifications"
            :key="tip.id"
            class="notification"
            :class="tip.type"
          >
            <p class="text-sm font-medium">{{ tip.title }}</p>
            <p class="mt-1 text-xs">{{ tip.message }}</p>
          </div>
        </transition-group>
      </section>
    </main>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref } from 'vue'
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
  }
}

onMounted(async () => {
  await loadStatus()
})
</script>
