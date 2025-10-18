<template>
  <div class="app-shell">
    <div class="app-background" aria-hidden="true">
      <div class="app-grid"></div>
      <div class="app-glow app-glow--primary"></div>
      <div class="app-glow app-glow--secondary"></div>
    </div>

    <TitleBar class="relative z-20" />

    <main class="app-main relative z-10">
      <section class="fusion-panel animate-slide-up">
        <header class="panel-header">
          <span class="panel-badge">Environment Pilot</span>
          <h1 class="panel-title">Python 环境初始化助手</h1>
          <p class="panel-subtitle">在当前目录快速检测与初始化 Python 运行环境，一键完成变量配置与镜像同步。</p>
        </header>

        <div class="status-wrapper" role="status" aria-live="polite">
          <div v-if="loading" class="status-grid">
            <div v-for="placeholder in 5" :key="`placeholder-${placeholder}`" class="status-card skeleton-card"></div>
          </div>
          <div v-else-if="statusItems.length > 0" class="status-grid">
            <article
              v-for="item in statusItems"
              :key="item.label"
              class="status-card"
              :class="item.tone"
            >
              <div class="status-card__beam"></div>
              <div class="status-card__icon" :class="item.tone">
                <component :is="item.icon" class="w-5 h-5" />
              </div>
              <div class="status-card__content">
                <p class="status-card__title">{{ item.label }}</p>
                <p class="status-card__message">{{ item.status.message }}</p>
                <p v-if="item.status.detail" class="status-card__detail">{{ item.status.detail }}</p>
              </div>
            </article>
          </div>
          <p v-else class="status-empty">暂未获取到环境状态，请稍后重试或检查控制台日志。</p>
        </div>

        <section class="panel-section">
          <div class="panel-section__header">
            <h2 class="section-title">镜像加速配置</h2>
            <p class="section-description">选择一个常用的国内镜像，加速依赖下载并自动写入 trusted-host。</p>
          </div>

          <div class="mirror-control">
            <select
              v-model="selectedMirror"
              class="input-apple"
              :disabled="initializing || loading"
            >
              <option
                v-for="mirror in mirrorOptions"
                :key="mirror.value"
                :value="mirror.value"
              >
                {{ mirror.label }} · {{ mirror.value }}
              </option>
            </select>

            <button
              class="btn-apple btn-primary"
              :disabled="initializing || loading"
              @click="handleInitialize"
            >
              <span v-if="initializing" class="btn-primary__loading">
                <svg class="w-4 h-4 animate-spin" viewBox="0 0 24 24" fill="none">
                  <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
                  <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8v4a4 4 0 00-4 4H4z" />
                </svg>
                <span>正在初始化</span>
              </span>
              <span v-else>写入镜像配置</span>
            </button>
          </div>

          <div v-if="activeMirror" class="trusted-hosts">
            <span class="trusted-hosts__label">Trusted Host</span>
            <ul class="trusted-hosts__list">
              <li
                v-for="host in activeMirror.trustedHosts"
                :key="host"
                class="trusted-hosts__chip"
              >
                {{ host }}
              </li>
            </ul>
          </div>
        </section>

        <section class="panel-section">
          <div class="panel-section__header">
            <h2 class="section-title">当前环境速览</h2>
            <p class="section-description">实时展示 Python 与 Pip 的路径信息，确保配置一目了然。</p>
          </div>

          <div class="meta-grid">
            <div class="meta-card">
              <span class="meta-card__label">Python 路径</span>
              <p class="meta-card__value">{{ status?.pythonPath ?? '未检测到' }}</p>
            </div>
            <div class="meta-card">
              <span class="meta-card__label">Scripts 目录</span>
              <p class="meta-card__value">{{ status?.scriptsPath ?? '未检测到' }}</p>
            </div>
            <div class="meta-card">
              <span class="meta-card__label">当前镜像源</span>
              <p class="meta-card__value">{{ status?.pipMirrorConfigured.currentMirror ?? '未检测到' }}</p>
            </div>
          </div>
        </section>

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
import { computed, onMounted, reactive, ref, type Component } from 'vue'
import { CheckCircleIcon, ExclamationTriangleIcon } from '@heroicons/vue/24/outline'
import TitleBar from './components/TitleBar.vue'
import {
  fetchPythonEnvStatus,
  initializePythonEnvironment,
  type PythonEnvStatus,
  type StatusLine,
  type MirrorOption,
} from './services/pythonEnv'

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

const mirrorOptions = computed<MirrorOption[]>(
  () =>
    status.value?.mirrorCandidates ?? [
      {
        label: '清华大学 TUNA',
        value: 'https://pypi.tuna.tsinghua.edu.cn/simple',
        trustedHosts: ['pypi.tuna.tsinghua.edu.cn'],
      },
    ],
)

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

interface StatusItem {
  label: string
  status: StatusLine
  tone: 'positive' | 'alert'
  icon: Component
}

const statusItems = computed<StatusItem[]>(() =>
  statusLines.value.map((item) => ({
    ...item,
    tone: item.status.ok ? 'positive' : 'alert',
    icon: item.status.ok ? CheckCircleIcon : ExclamationTriangleIcon,
  })),
)

const activeMirror = computed(() => mirrorOptions.value.find((item) => item.value === selectedMirror.value) ?? null)

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
