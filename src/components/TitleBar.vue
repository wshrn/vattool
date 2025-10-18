<template>
  <div
    class="app-title-bar"
    data-tauri-drag-region
    @mousedown="startDragging"
  >
    <div class="title-brand pointer-events-none">
      <span class="title-brand__glow" aria-hidden="true"></span>
      <span class="title-brand__badge" aria-hidden="true">知</span>
      <span class="title-brand__label">知攻系统</span>
    </div>
    <div
      class="title-actions"
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
