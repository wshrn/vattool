<template>
  <div class="flex flex-col h-screen min-h-screen bg-gray-50 dark:bg-gray-950 text-gray-900 dark:text-gray-50 border border-gray-200 dark:border-gray-800">
    <TitleBar />
    <main class="flex-1 overflow-hidden bg-gray-50 dark:bg-gray-950">
      <router-view />
    </main>
  </div>
</template>

<script setup lang="ts">
import TitleBar from '@/components/common/TitleBar.vue'
</script>
