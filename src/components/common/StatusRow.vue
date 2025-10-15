<template>
  <div
    class="result-item"
    :class="status ? 'success' : 'warning'"
  >
    <div class="flex items-center justify-between">
      <div class="flex items-center gap-3">
        <span class="flex items-center justify-center w-10 h-10 rounded-2xl bg-apple-blue/10 text-apple-blue dark:bg-apple-blue/20">
          <component :is="iconComponent" class="w-5 h-5" />
        </span>
        <div>
          <p class="text-base font-medium text-gray-900 dark:text-gray-100">{{ title }}</p>
          <p class="text-sm text-gray-600 dark:text-gray-300">{{ description }}</p>
        </div>
      </div>
      <span
        class="px-3 py-1 rounded-full text-sm font-medium"
        :class="status ? 'bg-apple-green/10 text-apple-green' : 'bg-yellow-500/10 text-yellow-600 dark:text-yellow-300'"
      >
        {{ status ? '已完成' : '待处理' }}
      </span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { BeakerIcon, CloudIcon, CpuChipIcon, FolderIcon, LinkIcon } from '@heroicons/vue/24/outline'

type IconType = 'chip' | 'folder' | 'link' | 'beaker' | 'cloud'

const props = defineProps<{
  icon: IconType
  title: string
  status: boolean
  description: string
}>()

const iconComponent = computed(() => {
  switch (props.icon) {
    case 'cloud':
      return CloudIcon
    case 'folder':
      return FolderIcon
    case 'link':
      return LinkIcon
    case 'beaker':
      return BeakerIcon
    default:
      return CpuChipIcon
  }
})
</script>
