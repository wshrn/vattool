<template>
  <div
    class="tech-titlebar flex h-11 w-full items-center justify-between px-4 select-none cursor-move"
    data-tauri-drag-region
    @mousedown="startDragging"
  >
    <div class="pointer-events-none flex items-center gap-3 text-xs font-medium text-slate-200">
      <span class="titlebar-logo" aria-hidden="true">知</span>
      <div class="leading-tight">
        <p class="titlebar-subtitle">VATTOOL</p>
        <p class="text-xs font-semibold text-white">知攻系统</p>
      </div>
    </div>
    <div
      class="flex items-center gap-2 pointer-events-auto"
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
