<template>
  <div class="h-screen flex flex-col bg-[var(--app-surface-bg)] text-gray-900 dark:text-gray-100 transition-colors">
    <TitleBar />
    <main class="flex-1 overflow-y-auto scrollbar-thin">
      <RouterView />
    </main>
  </div>
</template>

<script setup lang="ts">
import { RouterView } from 'vue-router'
import TitleBar from '@/components/TitleBar.vue'
</script>
