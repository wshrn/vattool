<template>
  <div ref="rootEl" class="titlebar" data-tauri-drag-region @mousedown="startDragging">
    <div class="titlebar__identity pointer-events-none">
      <span class="titlebar__badge" aria-hidden="true">知</span>
      <span>知攻系统 · Env Studio</span>
    </div>
    <div class="titlebar__actions pointer-events-auto" data-tauri-drag-region="false" @mousedown.stop>
      <ThemeToggle />
      <WindowControls />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'
import ThemeToggle from './ThemeToggle.vue'
import WindowControls from './WindowControls.vue'

const appWindow = getCurrentWindow()
const rootEl = ref<HTMLElement | null>(null)

const startDragging = async (event: MouseEvent) => {
  if (event.button === 0) {
    try {
      await appWindow.startDragging()
    } catch (error) {
      console.error('Failed to start dragging:', error)
    }
  }
}

defineExpose({
  rootEl,
})
</script>
