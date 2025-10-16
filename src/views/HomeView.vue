<template>
  <div class="min-h-full bg-[var(--app-surface-bg)] px-6 py-8 flex justify-center">
    <div class="w-full max-w-3xl card-apple p-8 space-y-6 animate-fade-in">
      <header class="space-y-2">
        <h1 class="text-2xl font-semibold text-gray-900 dark:text-gray-100">Python3 环境切换助手</h1>
        <p class="text-sm text-gray-500 dark:text-gray-400">自动检测并初始化与当前目录同级的 Python 可执行文件环境变量。</p>
      </header>

      <section class="space-y-3">
        <StatusIndicator
          v-for="line in statusLines"
          :key="line.label"
          :label="line.label"
          :message="line.message"
          :status="line.level"
        />
      </section>

      <section class="space-y-3">
        <label class="block text-sm font-medium text-gray-700 dark:text-gray-200">选择 Python 镜像源</label>
        <div class="relative">
          <select
            v-model="selectedMirror"
            class="input-apple pr-10"
            :disabled="!status?.runningOnWindows"
          >
            <option
              v-for="mirror in status?.availableMirrors ?? []"
              :key="mirror.id"
              :value="mirror.id"
            >
              {{ mirror.label }}
            </option>
          </select>
          <span class="absolute inset-y-0 right-0 flex items-center pr-3 pointer-events-none text-gray-400 dark:text-gray-500">
            ▼
          </span>
        </div>
      </section>

      <section class="space-y-2">
        <button
          class="btn-primary w-full"
          :disabled="isInitializing || !status?.runningOnWindows"
          @click="handleInitialize"
        >
          <span v-if="isInitializing" class="flex items-center justify-center space-x-2">
            <svg class="w-5 h-5 text-white loading-spinner" viewBox="0 0 24 24" fill="none">
              <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
              <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8v4a4 4 0 00-4 4H4z"></path>
            </svg>
            <span>初始化中...</span>
          </span>
          <span v-else>开始初始化</span>
        </button>
        <p v-if="!status?.runningOnWindows" class="text-xs text-apple-red">
          当前仅支持 Windows 平台的自动化初始化。
        </p>
        <p v-if="feedback" :class="['text-sm', feedback.success ? 'text-apple-green' : 'text-apple-red']">
          {{ feedback.message }}
        </p>
      </section>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import StatusIndicator from '@/components/StatusIndicator.vue'
import { fetchPythonEnvironmentStatus, initializePythonEnvironment } from '@/api/python'
import type { PythonEnvironmentStatus } from '@/api/python'

const status = ref<PythonEnvironmentStatus | null>(null)
const isInitializing = ref(false)
const selectedMirror = ref('')
const feedback = ref<{ success: boolean; message: string } | null>(null)

const statusLines = computed(() => status.value?.statusLines ?? [])

const refreshStatus = async () => {
  try {
    const result = await fetchPythonEnvironmentStatus()
    status.value = result
    selectedMirror.value = result.selectedMirror
  } catch (error) {
    console.error('获取环境状态失败', error)
  }
}

const handleInitialize = async () => {
  if (!status.value) {
    return
  }

  isInitializing.value = true
  feedback.value = null

  try {
    const result = await initializePythonEnvironment(selectedMirror.value)
    feedback.value = { success: result.success, message: result.message }
    if (result.status) {
      status.value = result.status
      selectedMirror.value = result.status.selectedMirror
    } else {
      await refreshStatus()
    }
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error)
    feedback.value = { success: false, message: `初始化失败：${message}` }
  } finally {
    isInitializing.value = false
  }
}

onMounted(() => {
  void refreshStatus()
})
</script>
