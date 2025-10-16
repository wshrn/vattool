<template>
  <div class="h-screen flex flex-col bg-[var(--app-surface-bg)] text-gray-900 dark:text-gray-100">
    <TitleBar />
    <main class="flex-1 overflow-y-auto scrollbar-thin">
      <div class="max-w-4xl mx-auto px-6 py-8 space-y-6">
        <section class="card-apple p-6 space-y-6">
          <header class="flex items-start justify-between">
            <div>
              <h1 class="text-2xl font-semibold text-gray-900 dark:text-gray-100">Python 环境快速巡检</h1>
              <p class="text-sm text-gray-500 dark:text-gray-400 mt-2">
                工具将自动检测与初始化当前目录下 Python 可执行文件及相关环境变量配置，请确保与 python.exe 同目录运行。
              </p>
            </div>
          </header>

          <div class="grid gap-3">
            <StatusItem
              label="Python 可执行文件检测"
              :active="status?.pythonExists ?? false"
              positive-label="已检测到 python.exe"
              negative-label="未发现 python.exe"
              :state="status?.pythonExists ? 'success' : 'error'"
              :description="status?.pythonExists ? status?.pythonPath : '请确认工具与 python.exe 位于同一目录'"
            />
            <StatusItem
              label="Python3 环境变量配置"
              :active="status?.pythonEnvVarSet ?? false"
              positive-label="已写入 PYTHON3 环境变量"
              negative-label="待创建 PYTHON3 变量"
              :state="status?.pythonEnvVarSet ? 'success' : 'warning'"
              :description="status?.pythonEnvVarSet ? `PYTHON3 = ${pythonDirectory || '未知路径'}` : '初始化时将自动创建 PYTHON3 环境变量'"
            />
            <StatusItem
              label="Python 主目录加入 PATH"
              :active="status?.pythonPathSet ?? false"
              positive-label="PATH 已包含 Python"
              negative-label="PATH 未包含 Python"
              :state="status?.pythonPathSet ? 'success' : 'warning'"
              :description="status?.pythonPathSet
                ? `PATH 已包含 ${pythonDirectory || 'Python 目录'}`
                : pythonDirectory
                  ? `初始化将添加 ${pythonDirectory} 至 PATH`
                  : '初始化将添加 Python 目录至 PATH'"
            />
            <StatusItem
              label="Scripts 目录加入 PATH"
              :active="status?.scriptsPathSet ?? false"
              positive-label="PATH 已包含 Scripts"
              negative-label="PATH 未包含 Scripts"
              :state="status?.scriptsPathSet ? 'success' : 'warning'"
              :description="status?.scriptsPath ? `当前 Scripts 路径：${status?.scriptsPath}` : '初始化将添加 Scripts 目录至 PATH'"
            />
            <StatusItem
              label="pip 国内镜像配置"
              :active="status?.mirrorConfigured ?? false"
              :positive-label="status?.mirrorConfigured ? `已配置 ${status?.mirrorName || '国内镜像'}` : '国内镜像未配置'"
              negative-label="未配置国内镜像"
              :state="status?.mirrorConfigured ? 'success' : 'info'"
              :description="status?.mirrorConfigured ? `当前镜像：${status?.mirrorName}` : '推荐使用国内镜像以提升下载速度'"
            />
          </div>
        </section>

        <section class="card-apple p-6 space-y-4">
          <div class="space-y-2">
            <h2 class="text-lg font-semibold text-gray-900 dark:text-gray-100">pip 国内源选择</h2>
            <p class="text-sm text-gray-500 dark:text-gray-400">
              请选择需要写入的全局 pip 镜像源，默认已选择清华源，初始化过程会写入 %APPDATA%\pip\pip.ini 并创建所需目录。
            </p>
            <div class="relative">
              <select
                v-model="selectedMirror"
                class="input-apple appearance-none pr-10"
              >
                <option
                  v-for="mirror in mirrors"
                  :key="mirror.value"
                  :value="mirror.value"
                >
                  {{ mirror.label }}
                </option>
              </select>
              <div class="absolute inset-y-0 right-0 flex items-center pr-3 pointer-events-none text-gray-400">
                <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
                </svg>
              </div>
            </div>
          </div>

          <div class="flex items-center justify-between">
            <div class="text-sm text-gray-500 dark:text-gray-400">
              <p>选择的镜像将写入 pip.ini 的 <code>index-url</code> 与 <code>trusted-host</code> 配置。</p>
              <ul v-if="statusDetails.length" class="mt-1 text-xs opacity-80 space-y-1">
                <li v-for="(detail, index) in statusDetails" :key="`${index}-${detail}`">{{ detail }}</li>
              </ul>
            </div>
            <button
              class="btn-primary flex items-center space-x-2"
              :disabled="isInitializing"
              @click="handleInitialize"
            >
              <span v-if="!isInitializing">开始初始化</span>
              <span v-else class="flex items-center space-x-2">
                <svg class="w-4 h-4 loading-spinner" viewBox="0 0 24 24" fill="none">
                  <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                  <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8v4a4 4 0 00-4 4H4z"></path>
                </svg>
                <span>正在初始化...</span>
              </span>
            </button>
          </div>

          <transition name="fade">
            <div v-if="feedback" :class="['notification', feedback.success ? 'success' : 'error']">
              <p class="font-medium">{{ feedback.message }}</p>
            </div>
          </transition>
        </section>
      </div>
    </main>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref, watch } from 'vue'
import TitleBar from '@/components/TitleBar.vue'
import StatusItem from '@/components/StatusItem.vue'
import { fetchPythonStatus, initializePythonEnvironment } from '@/api/python'
import type { PythonEnvironmentStatus } from '@/types/python'

const status = ref<PythonEnvironmentStatus | null>(null)
const isInitializing = ref(false)
const feedback = ref<{ success: boolean; message: string } | null>(null)

const pythonDirectory = computed(() => {
  const path = status.value?.pythonPath
  if (!path) {
    return ''
  }
  const separator = path.includes('\\') && !path.includes('/') ? '\\' : '/'
  const segments = path.split(separator)
  if (segments.length <= 1) {
    return path
  }
  segments.pop()
  return segments.join(separator)
})

const statusDetails = computed(() => {
  if (!status.value?.details) {
    return [] as string[]
  }
  return status.value.details
    .split('\n')
    .map(item => item.trim())
    .filter(item => item.length > 0)
})

const mirrors = reactive([
  { label: '清华大学 TUNA 镜像', value: 'https://pypi.tuna.tsinghua.edu.cn/simple', host: 'pypi.tuna.tsinghua.edu.cn' },
  { label: '阿里云镜像', value: 'https://mirrors.aliyun.com/pypi/simple/', host: 'mirrors.aliyun.com' },
  { label: '中国科学技术大学镜像', value: 'https://pypi.mirrors.ustc.edu.cn/simple', host: 'pypi.mirrors.ustc.edu.cn' },
  { label: '华为云镜像', value: 'https://repo.huaweicloud.com/repository/pypi/simple', host: 'repo.huaweicloud.com' },
  { label: '腾讯云镜像', value: 'https://mirrors.cloud.tencent.com/pypi/simple', host: 'mirrors.cloud.tencent.com' }
])

const selectedMirror = ref(mirrors[0].value)

const refreshStatus = async () => {
  try {
    const result = await fetchPythonStatus()
    if (result) {
      status.value = result
      if (result.mirrorConfigured) {
        const matched = mirrors.find(item => result.mirrorName.includes(item.host))
        if (matched) {
          selectedMirror.value = matched.value
        }
      }
    }
  } catch (error) {
    console.error('获取 Python 环境状态失败', error)
  }
}

onMounted(async () => {
  await refreshStatus()
})

watch(selectedMirror, () => {
  feedback.value = null
})

const handleInitialize = async () => {
  if (isInitializing.value) {
    return
  }
  feedback.value = null
  isInitializing.value = true
  try {
    const result = await initializePythonEnvironment(selectedMirror.value)
    if (result) {
      status.value = result.status
      feedback.value = {
        success: result.success,
        message: result.message
      }
    } else {
      feedback.value = {
        success: false,
        message: '当前环境为预览模式，未执行初始化。'
      }
    }
  } catch (error) {
    const message = error instanceof Error ? error.message : '初始化失败，请查看日志。'
    feedback.value = {
      success: false,
      message
    }
  } finally {
    isInitializing.value = false
    await refreshStatus()
  }
}

watch(status, newStatus => {
  if (newStatus?.mirrorConfigured) {
    const matched = mirrors.find(item => newStatus.mirrorName.includes(item.host))
    if (matched) {
      selectedMirror.value = matched.value
    }
  }
})
</script>

<style scoped>
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.3s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
