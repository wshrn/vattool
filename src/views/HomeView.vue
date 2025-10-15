<template>
  <div class="flex flex-col gap-6 p-8">
    <section class="card-apple p-6 animate-fade-in">
      <header class="flex items-center justify-between mb-6">
        <div>
          <h1 class="text-2xl font-semibold text-gray-900 dark:text-gray-50">Python 环境检查</h1>
          <p class="text-sm text-gray-500 dark:text-gray-400 mt-1">快速了解当前目录中的 Python 解释器与环境变量配置状态。</p>
        </div>
        <ThemeToggle />
      </header>

      <div class="space-y-3">
        <StatusRow
          icon="chip"
          title="Python 可执行文件"
          :status="status.pythonFound"
          :description="status.pythonFound ? status.pythonPath ?? '检测到 Python 执行文件' : '未在当前目录发现 python.exe 或 python3.exe'"
        />
        <StatusRow
          icon="folder"
          title="python3 环境变量"
          :status="status.python3EnvExists"
          :description="status.python3EnvExists ? `python3 = ${status.python3EnvValue}` : '未找到名为 python3 的用户环境变量'"
        />
        <StatusRow
          icon="link"
          title="PATH 包含 Python"
          :status="status.pathHasPython"
          description="检查 PATH 是否包含 python3 目录或变量引用"
        />
        <StatusRow
          icon="beaker"
          title="PATH 包含 Scripts"
          :status="status.pathHasScripts"
          description="检查 PATH 是否包含 python3\\Scripts"
        />
        <StatusRow
          icon="cloud"
          title="PIP 国内镜像"
          :status="status.pipConfigured"
          :description="status.pipConfigured ? `当前镜像：${status.pipIndexUrl ?? '已配置'}` : '未检测到 pip 全局国内镜像配置'"
        />
      </div>

      <div v-if="status.messages.length" class="mt-6 space-y-2">
        <p class="text-sm font-medium text-gray-600 dark:text-gray-300">提示</p>
        <ul class="space-y-2">
          <li
            v-for="message in status.messages"
            :key="message"
            class="result-item info text-sm text-gray-600 dark:text-gray-200"
          >
            {{ message }}
          </li>
        </ul>
      </div>
    </section>

    <section class="card-apple p-6 animate-slide-up">
      <h2 class="text-xl font-semibold text-gray-900 dark:text-gray-50 mb-4">国内镜像选择</h2>
      <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
        <label class="text-sm font-medium text-gray-600 dark:text-gray-300">选择 PIP 镜像</label>
        <select
          v-model="selectedMirror"
          class="input-apple"
        >
          <option v-for="mirror in pipMirrors" :key="mirror.value" :value="mirror.value">
            {{ mirror.label }}
          </option>
        </select>
      </div>

      <button
        class="btn-primary mt-6 self-start"
        :disabled="loading"
        @click="handleInitialize"
      >
        <span v-if="!loading">开始初始化</span>
        <span v-else class="flex items-center gap-2">
          <svg class="w-4 h-4 loading-spinner" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle class="opacity-25" cx="12" cy="12" r="10" />
            <path class="opacity-75" d="M12 2a10 10 0 0 1 10 10" />
          </svg>
          执行中...
        </span>
      </button>
    </section>
  </div>
</template>

<script setup lang="ts">
import { onMounted, reactive, ref } from 'vue'
import ThemeToggle from '@/components/common/ThemeToggle.vue'
import StatusRow from '@/components/common/StatusRow.vue'
import { ToolboxAPI, type EnvironmentStatus } from '@/api'

const status = reactive<EnvironmentStatus>({
  pythonFound: false,
  pythonPath: '',
  python3EnvExists: false,
  python3EnvValue: '',
  pathHasPython: false,
  pathHasScripts: false,
  pipConfigured: false,
  pipIndexUrl: '',
  messages: []
})

const selectedMirror = ref('tsinghua')
const loading = ref(false)

const pipMirrors = [
  { label: '清华大学 (默认)', value: 'tsinghua' },
  { label: '中国科技大学', value: 'ustc' },
  { label: '阿里云', value: 'aliyun' },
  { label: '华为云', value: 'huawei' },
  { label: '腾讯云', value: 'tencent' }
]

const loadStatus = async () => {
  try {
    const data = await ToolboxAPI.getEnvironmentStatus()
    Object.assign(status, data)
  } catch (error) {
    console.error('加载环境状态失败', error)
    status.messages = ['无法获取环境状态，请确认后端命令可用。']
  }
}

const handleInitialize = async () => {
  loading.value = true
  try {
    const result = await ToolboxAPI.initializeEnvironment(selectedMirror.value)
    if (result.status) {
      Object.assign(status, result.status)
    }
    status.messages = [result.message, ...(status.messages ?? [])]
  } catch (error) {
    console.error('初始化失败', error)
    status.messages = ['初始化失败，请检查日志。']
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  void loadStatus()
})
</script>
