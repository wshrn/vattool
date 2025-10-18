<template>
  <div class="flex items-center gap-1.5">
    <button
      type="button"
      class="control-button"
      aria-label="最小化窗口"
      @click.stop.prevent="minimize"
      @mousedown.stop
    >
      <MinusIcon />
    </button>
    <button
      type="button"
      class="control-button"
      aria-label="切换窗口最大化"
      @click.stop.prevent="toggleMaximize"
      @mousedown.stop
    >
      <Squares2X2Icon
        v-if="!isMaximized"
      />
      <Squares2X2Icon
        v-else
        class="rotate-45"
      />
    </button>
    <button
      type="button"
      class="control-button control-button--danger"
      aria-label="关闭窗口"
      @click.stop.prevent="closeWindow"
      @mousedown.stop
    >
      <XMarkIcon />
    </button>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import type { UnlistenFn } from '@tauri-apps/api/event'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { MinusIcon, Squares2X2Icon, XMarkIcon } from '@heroicons/vue/24/outline'

const appWindow = getCurrentWindow()
const isMaximized = ref(false)
const listeners: UnlistenFn[] = []

type WindowApiWithOptionalEvents = typeof appWindow & {
  onMaximize?: (handler: () => void) => Promise<UnlistenFn>
  onUnmaximize?: (handler: () => void) => Promise<UnlistenFn>
}

const windowApi = appWindow as WindowApiWithOptionalEvents

const refreshState = async () => {
  try {
    isMaximized.value = await appWindow.isMaximized()
  } catch (error) {
    console.error('Failed to determine maximize state:', error)
  }
}

onMounted(async () => {
  await refreshState()

  try {
    listeners.push(await appWindow.onResized(refreshState))

    if (typeof windowApi.onMaximize === 'function') {
      listeners.push(
        await windowApi.onMaximize(() => {
          isMaximized.value = true
        }),
      )
    }

    if (typeof windowApi.onUnmaximize === 'function') {
      listeners.push(
        await windowApi.onUnmaximize(() => {
          isMaximized.value = false
        }),
      )
    }
  } catch (error) {
    console.error('Failed to register window listeners:', error)
  }
})

onUnmounted(() => {
  listeners.forEach((unlisten) => {
    try {
      unlisten()
    } catch (error) {
      console.warn('Failed to clean up window listener:', error)
    }
  })
  listeners.length = 0
})

const minimize = async () => {
  try {
    await appWindow.minimize()
  } catch (error) {
    console.error('Failed to minimize window:', error)
  }
}

const toggleMaximize = async () => {
  try {
    const maximized = await appWindow.isMaximized()
    if (maximized) {
      await appWindow.unmaximize()
    } else {
      await appWindow.maximize()
    }
    await refreshState()
  } catch (error) {
    console.error('Failed to toggle maximize:', error)
  }
}

const closeWindow = async () => {
  try {
    await appWindow.close()
  } catch (error) {
    console.error('Failed to close window:', error)
  }
}
</script>
