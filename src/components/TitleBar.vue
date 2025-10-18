<template>
  <div
    class="titlebar-surface relative flex h-9 w-full items-center justify-between overflow-hidden px-3 text-[11px] font-medium uppercase tracking-[0.2em] text-slate-200"
    data-tauri-drag-region
    @mousedown="startDragging"
  >
    <div class="pointer-events-none flex items-center gap-2 text-[10px]">
      <span
        class="inline-flex h-5 w-5 items-center justify-center rounded-full bg-gradient-to-br from-sky-400 to-blue-600 text-[11px] font-semibold text-white shadow-[0_0_12px_rgba(56,189,248,0.45)]"
        aria-hidden="true"
      >
        知
      </span>
      <span class="tracking-[0.28em] text-slate-300">知攻系统</span>
    </div>
    <div
      class="pointer-events-auto flex items-center gap-2"
      data-tauri-drag-region="false"
      @mousedown.stop
    >
      <ThemeToggle />
      <WindowControls />
    </div>
    <div class="pointer-events-none absolute inset-0 -z-10">
      <div class="absolute inset-0 bg-[radial-gradient(circle_at_top,_rgba(56,189,248,0.2),_transparent_70%)]"></div>
      <div class="absolute inset-x-0 bottom-0 h-px bg-gradient-to-r from-transparent via-sky-400/60 to-transparent"></div>
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
