<template>
  <div class="flex items-center space-x-3">
    <button
      class="p-2 rounded-lg bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors"
      :title="`当前主题：${themeLabel}`"
      @click="toggleTheme"
    >
      <MoonIcon v-if="isDark" class="w-5 h-5" />
      <SunIcon v-else class="w-5 h-5" />
    </button>
    <span class="text-xs text-gray-500 dark:text-gray-400">{{ themeLabel }}</span>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { storeToRefs } from 'pinia'
import { MoonIcon, SunIcon } from '@heroicons/vue/24/outline'
import { useThemeStore } from '@/stores/theme'

type ThemeMode = 'light' | 'dark' | 'auto'

const themeStore = useThemeStore()
const { isDark, theme } = storeToRefs(themeStore)

const themeLabel = computed(() => {
  switch (theme.value) {
    case 'dark':
      return '深色模式'
    case 'light':
      return '浅色模式'
    default:
      return '跟随系统'
  }
})

const toggleTheme = () => {
  const modes: ThemeMode[] = ['light', 'dark', 'auto']
  const currentIndex = modes.indexOf(theme.value)
  const next = modes[(currentIndex + 1) % modes.length]
  void themeStore.setTheme(next)
}
</script>
