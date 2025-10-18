<template>
  <div
    class="titlebar-surface flex h-12 w-full items-center justify-between px-4 select-none cursor-move"
    data-tauri-drag-region
    @mousedown="startDragging"
  >
    <div class="pointer-events-none flex items-center space-x-3 text-xs font-semibold uppercase tracking-[0.4em] text-slate-200/90">
      <span class="glow-dot" aria-hidden="true" />
      <span>知攻系统</span>
    </div>
    <div
      class="flex items-center space-x-2 pointer-events-auto"
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
