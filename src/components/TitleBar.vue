<template>
  <div
    class="flex items-center justify-between w-full h-9 bg-white dark:bg-gray-900 border-b border-gray-200 dark:border-gray-800 px-3 select-none cursor-move"
    data-tauri-drag-region
  >
    <div class="flex items-center space-x-2 text-xs font-medium text-gray-700 dark:text-gray-200 pointer-events-none">
      <span
        class="flex items-center justify-center w-5 h-5 text-[11px] font-semibold text-white bg-apple-blue rounded"
        aria-hidden="true"
      >
        知
      </span>
      <span>知攻系统</span>
    </div>
    <div
      class="flex items-center space-x-2 pointer-events-auto"
      data-tauri-drag-region="false"
    >
      <ThemeToggle />
      <WindowControls />
    </div>
  </div>
</template>

<script setup lang="ts">
import ThemeToggle from './ThemeToggle.vue'
import WindowControls from './WindowControls.vue'
</script>
