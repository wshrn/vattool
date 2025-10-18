<template>
  <div class="relative min-h-screen overflow-hidden bg-slate-950 text-slate-100 transition-colors duration-500 ease-apple tech-background">
    <div class="pointer-events-none absolute inset-0">
      <div class="absolute -left-40 -top-32 h-96 w-96 rounded-full bg-cyan-500/30 blur-3xl" />
      <div class="absolute bottom-0 right-0 h-[28rem] w-[28rem] translate-x-1/3 translate-y-1/3 rounded-full bg-purple-500/25 blur-3xl" />
      <div class="absolute inset-0 bg-[radial-gradient(circle_at_top,_rgba(34,211,238,0.08),_transparent_60%)]" />
    </div>

    <TitleBar />

    <main class="relative mx-auto max-w-5xl px-6 py-16">
      <section class="card-apple relative overflow-hidden p-10">
        <div class="absolute inset-0 pointer-events-none">
          <div class="absolute inset-0 bg-gradient-to-br from-cyan-500/10 via-transparent to-purple-500/10 opacity-70" />
          <div class="absolute -inset-px rounded-[1.85rem] border border-white/10" />
        </div>

        <div class="relative space-y-8">
          <header class="space-y-4">
            <span class="inline-flex items-center gap-2 rounded-full border border-cyan-400/20 bg-cyan-500/10 px-4 py-1 text-[11px] font-medium tracking-[0.35em] text-cyan-200/80 uppercase">
              Env Sync
            </span>
            <h1 class="text-3xl font-semibold leading-tight md:text-4xl">
              Python 环境初始化助手
            </h1>
            <p class="max-w-3xl text-sm leading-relaxed text-slate-300/80">
              在当前目录快速检测并初始化 Python 运行环境，自动校验 PATH、Scripts、pip 国内镜像等配置，并在初始化过程中提供实时反馈。
            </p>
          </header>

          <div class="grid gap-3 sm:grid-cols-2">
            <template v-if="loading">
              <div
                v-for="index in 5"
                :key="`skeleton-${index}`"
                class="result-item loading"
              >
                <div class="skeleton-bar h-3 w-24" />
                <div class="skeleton-bar h-2 w-3/4" />
                <div class="skeleton-bar h-2 w-1/2" />
              </div>
            </template>
            <template v-else>
              <div
                v-for="item in statusLines"
                :key="item.label"
                class="result-item"
                :class="item.status.ok ? 'success' : 'error'"
              >
                <div class="flex items-start justify-between gap-4">
                  <div class="space-y-2">
                    <p class="text-sm font-semibold tracking-wide text-slate-100">
                      {{ item.label }}
                    </p>
                    <p class="text-xs text-slate-300/80 whitespace-pre-line">
                      {{ item.status.message }}
                    </p>
                    <p
                      v-if="item.status.detail"
                      class="text-xs text-slate-400/80"
                    >
                      {{ item.status.detail }}
                    </p>
                  </div>
                  <span class="status-indicator" :class="item.status.ok ? 'success' : 'error'">
                    <svg v-if="item.status.ok" class="h-4 w-4" viewBox="0 0 24 24" fill="none">
                      <path
                        d="M5 13l4 4L19 7"
                        stroke="currentColor"
                        stroke-width="1.5"
                        stroke-linecap="round"
                        stroke-linejoin="round"
                      />
                    </svg>
                    <svg v-else class="h-4 w-4" viewBox="0 0 24 24" fill="none">
                      <path
                        d="M15 9l-6 6m0-6l6 6"
                        stroke="currentColor"
                        stroke-width="1.5"
                        stroke-linecap="round"
                      />
                    </svg>
                  </span>
                </div>
              </div>
            </template>
          </div>

          <div class="space-y-4">
            <div class="space-y-2">
              <label class="block text-sm font-semibold tracking-wide text-slate-100">选择默认 pip 国内镜像源</label>
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
              <p class="text-xs text-slate-400/80">
                将写入全局 <code>pip.ini</code> 配置，并自动加入对应的 trusted-host。
              </p>
            </div>

            <div class="info-panel grid gap-4 rounded-2xl border border-white/10 bg-white/5 p-4 text-xs text-slate-200/90 md:grid-cols-3">
              <div class="space-y-1">
                <p class="text-[11px] uppercase tracking-[0.2em] text-slate-400/70">Python</p>
                <p class="font-mono text-[13px]">
                  {{ status?.pythonPath ?? '未检测到' }}
                </p>
              </div>
              <div class="space-y-1">
                <p class="text-[11px] uppercase tracking-[0.2em] text-slate-400/70">Scripts</p>
                <p class="font-mono text-[13px]">
                  {{ status?.scriptsPath ?? '未检测到' }}
                </p>
              </div>
              <div class="space-y-1">
                <p class="text-[11px] uppercase tracking-[0.2em] text-slate-400/70">当前镜像</p>
                <p class="font-mono text-[13px]">
                  {{ status?.pipMirrorConfigured.currentMirror ?? '未检测到' }}
                </p>
              </div>
            </div>
          </div>

          <div class="flex flex-col items-start justify-between gap-4 sm:flex-row sm:items-center">
            <div class="text-xs text-slate-400/80">
              <p>初始化会在后台自动写入配置，操作完成后可立即使用。</p>
              <p>如需恢复默认镜像，可再次运行并选择其他源。</p>
            </div>
            <button
              class="btn-primary"
              :disabled="initializing || loading"
              @click="handleInitialize"
            >
              <span v-if="initializing" class="flex items-center gap-2">
                <svg class="h-4 w-4 animate-spin" viewBox="0 0 24 24" fill="none">
                  <circle class="opacity-30" cx="12" cy="12" r="9" stroke="currentColor" stroke-width="3" />
                  <path class="opacity-80" fill="currentColor" d="M12 3a9 9 0 018.66 6.5l-3.77 1A5 5 0 0012 7z" />
                </svg>
                <span>正在初始化...</span>
              </span>
              <span v-else class="flex items-center gap-2">
                <svg class="h-4 w-4" viewBox="0 0 24 24" fill="none">
                  <path
                    d="M4 12h6l2-3 2 6 2-3h4"
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

          <transition-group name="list" tag="div" class="space-y-2">
            <div
              v-for="tip in notifications"
              :key="tip.id"
              class="notification"
              :class="tip.type"
            >
              <p class="text-sm font-medium">{{ tip.title }}</p>
              <p class="text-xs mt-1 text-slate-200/90">{{ tip.message }}</p>
            </div>
          </transition-group>
        </div>
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
