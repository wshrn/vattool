<template>
  <div class="titlebar" data-tauri-drag-region @mousedown="startDragging">
    <div class="titlebar__brand" data-tauri-drag-region>
      <span class="titlebar__badge" aria-hidden="true">知</span>
      <div class="titlebar__text">
        <span class="titlebar__name">知攻系统</span>
        <span class="titlebar__caption">Python Env Assistant</span>
      </div>
    </div>
    <div class="titlebar__actions" data-tauri-drag-region="false" @mousedown.stop>
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
