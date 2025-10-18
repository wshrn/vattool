<template>
  <div
    class="app-titlebar"
    data-tauri-drag-region
    @mousedown="startDragging"
  >
    <div class="app-titlebar__brand pointer-events-none">
      <span class="app-titlebar__logo" aria-hidden="true">知</span>
      <span class="app-titlebar__name">知攻系统</span>
    </div>
    <div
      class="app-titlebar__controls pointer-events-auto"
      data-tauri-drag-region="false"
      @mousedown.stop
    >
      <ThemeToggle />
      <WindowControls />
    </div>
  </div>
</template>

<script setup lang="ts">
import { getCurrentWindow } from '@tauri-apps/api/window'
import ThemeToggle from './ThemeToggle.vue'
import WindowControls from './WindowControls.vue'

const appWindow = getCurrentWindow()

const startDragging = async (event: MouseEvent) => {
  if (event.button === 0) {
    try {
      await appWindow.startDragging()
    } catch (error) {
      console.error('Failed to start dragging:', error)
    }
  }
}
</script>
