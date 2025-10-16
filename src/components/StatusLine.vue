<template>
  <div :class="['result-item flex items-center justify-between gap-4', status.level]">
    <div class="flex items-center space-x-3">
      <component :is="iconComponent" class="w-5 h-5" />
      <div>
        <p class="text-sm font-medium">{{ title }}</p>
        <p class="text-xs text-gray-500 dark:text-gray-400">{{ status.message }}</p>
      </div>
    </div>
    <span
      :class="[
        'px-3 py-1 rounded-full text-xs font-semibold transition-colors duration-200',
        badgeClass,
      ]"
    >
      {{ status.ok ? '已完成' : '未完成' }}
    </span>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { BoltIcon, CommandLineIcon, GlobeAltIcon, CpuChipIcon, FolderIcon } from '@heroicons/vue/24/outline'

const props = defineProps({
  icon: {
    type: String,
    required: true,
  },
  title: {
    type: String,
    required: true,
  },
  status: {
    type: Object as () => { ok: boolean; message: string; level: 'success' | 'warning' | 'error' | 'info' },
    required: true,
  },
})

const iconComponent = computed(() => {
  switch (props.icon) {
    case 'cpu':
      return CpuChipIcon
    case 'variable':
      return BoltIcon
    case 'path':
      return FolderIcon
    case 'script':
      return CommandLineIcon
    case 'globe':
      return GlobeAltIcon
    default:
      return BoltIcon
  }
})

const badgeClass = computed(() => {
  if (props.status.ok) {
    return 'bg-green-100 text-green-700 dark:bg-green-900/40 dark:text-green-200'
  }
  switch (props.status.level) {
    case 'error':
      return 'bg-red-100 text-red-700 dark:bg-red-900/40 dark:text-red-200'
    case 'warning':
      return 'bg-yellow-100 text-yellow-700 dark:bg-yellow-900/40 dark:text-yellow-200'
    default:
      return 'bg-gray-100 text-gray-600 dark:bg-gray-700 dark:text-gray-300'
  }
})
</script>
