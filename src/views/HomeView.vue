<template>
  <div class="max-w-3xl mx-auto space-y-6">
    <section class="card-apple p-8 space-y-6">
      <header class="space-y-1">
        <h2 class="text-lg font-semibold">环境检测概览</h2>
        <p class="text-sm text-gray-500 dark:text-gray-400">
          检查当前目录下的 Python 可执行文件与环境变量配置情况。
        </p>
      </header>

      <div class="space-y-3">
        <StatusRow
          icon="cpu"
          :ok="status.pythonExecutableFound"
          :description="status.pythonExecutableFound ? `检测到 Python 可执行文件：${status.pythonExecutablePath}` : '未在当前目录找到 python.exe / python3.exe'"
          title="Python 可执行文件"
        />
        <StatusRow
          icon="variable"
          :ok="status.pythonEnvVarSet"
          :description="status.pythonEnvVarSet ? `python3 环境变量已指向：${status.pythonEnvVarValue}` : '未找到 python3 环境变量或指向错误'"
          title="python3 环境变量"
        />
        <StatusRow
          icon="path"
          :ok="status.pythonPathConfigured"
          :description="status.pythonPathConfigured ? 'Python 目录已加入 PATH' : 'PATH 中未包含 python3 变量或对应目录'"
          title="PATH 中的 Python"
        />
        <StatusRow
          icon="script"
          :ok="status.scriptsPathConfigured"
          :description="status.scriptsPathConfigured ? 'Scripts 目录已加入 PATH' : 'PATH 中未包含 Scripts 目录'"
          title="PATH 中的 Scripts"
        />
        <StatusRow
          icon="cloud"
          :ok="status.pipMirrorConfigured"
          :description="status.pipMirrorConfigured ? `当前 PIP 镜像：${status.pipMirrorUrl}` : '未检测到全球镜像配置，将使用默认源'"
          title="PIP 国内镜像"
        >
          <template #extra>
            <label class="flex items-center space-x-2 text-xs text-gray-500 dark:text-gray-400">
              <span>国内源</span>
              <select v-model="selectedMirror" class="input-apple h-9 text-sm max-w-xs">
                <option v-for="(option, key) in mirrorOptionsComputed" :key="key" :value="key">
                  {{ option.label }}
                </option>
              </select>
            </label>
          </template>
        </StatusRow>
      </div>

      <div class="pt-2">
        <button class="btn-primary" :disabled="loading" @click="handleInitialize">
          <span v-if="loading" class="flex items-center space-x-2">
            <svg class="w-4 h-4 loading-spinner" viewBox="0 0 24 24" fill="none">
              <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
              <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8v4a4 4 0 00-4 4H4z"></path>
            </svg>
            <span>正在初始化...</span>
          </span>
          <span v-else>开始初始化</span>
        </button>
      </div>
    </section>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref } from 'vue'
import StatusRow from '@/components/status/StatusRow.vue'
import { fetchPythonStatus, initializePythonEnvironment, mirrorOptions, type MirrorOptionKey } from '@/services/python'
import { useNotificationStore } from '@/stores/notification'

const notificationStore = useNotificationStore()

const status = reactive({
  pythonExecutableFound: false,
  pythonExecutablePath: undefined as string | undefined,
  pythonEnvVarSet: false,
  pythonEnvVarValue: undefined as string | undefined,
  pythonPathConfigured: false,
  scriptsPathConfigured: false,
  pipMirrorConfigured: false,
  pipMirrorUrl: undefined as string | undefined
})

const loading = ref(false)
const selectedMirror = ref<MirrorOptionKey>('tsinghua')

const mirrorOptionsComputed = computed(() => mirrorOptions)

const refreshStatus = async () => {
  try {
    const result = await fetchPythonStatus()
    Object.assign(status, result)
    if (result.pipMirrorUrl) {
      const entry = Object.entries(mirrorOptions).find(([, option]) => option.url === result.pipMirrorUrl)
      if (entry) {
        selectedMirror.value = entry[0] as MirrorOptionKey
      }
    }
  } catch (error) {
    console.error('Failed to fetch python status', error)
    notificationStore.push({
      title: '检测失败',
      message: error instanceof Error ? error.message : String(error),
      type: 'error'
    })
  }
}

onMounted(() => {
  void refreshStatus()
})

const handleInitialize = async () => {
  loading.value = true
  try {
    const result = await initializePythonEnvironment({ mirror: selectedMirror.value })
    Object.assign(status, result)
    notificationStore.push({ title: '初始化完成', message: 'Python 环境变量与镜像已更新。', type: 'success' })
  } catch (error) {
    console.error('Failed to initialize python environment', error)
    notificationStore.push({
      title: '初始化失败',
      message: error instanceof Error ? error.message : String(error),
      type: 'error'
    })
  } finally {
    loading.value = false
  }
}
</script>
