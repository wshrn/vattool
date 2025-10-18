<template>
  <div class="title-bar" data-tauri-drag-region @mousedown="startDragging">
    <div class="title-bar__brand pointer-events-none">
      <span class="title-bar__emblem" aria-hidden="true">知</span>
      <span>知攻系统</span>
    </div>
    <div class="title-bar__actions pointer-events-auto" data-tauri-drag-region="false" @mousedown.stop>
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
