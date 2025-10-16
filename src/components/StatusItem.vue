<template>
  <div :class="['result-item flex items-center justify-between', variantClass]">
    <div>
      <p class="text-sm font-medium text-gray-700 dark:text-gray-200">{{ label }}</p>
      <p v-if="description" class="text-xs text-gray-500 dark:text-gray-400 mt-1">{{ description }}</p>
    </div>
    <span :class="['px-3 py-1 rounded-full text-xs font-semibold', badgeClass]">
      {{ statusText }}
    </span>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

type StatusState = 'success' | 'warning' | 'error' | 'info'

const props = defineProps<{
  label: string
  active: boolean
  positiveLabel?: string
  negativeLabel?: string
  description?: string
  state?: StatusState
}>()

const variantClass = computed(() => {
  switch (props.state) {
    case 'success':
      return 'success'
    case 'warning':
      return 'warning'
    case 'error':
      return 'error'
    default:
      return 'info'
  }
})

const badgeClass = computed(() => {
  switch (props.state) {
    case 'success':
      return 'bg-green-100 text-green-700 dark:bg-green-900/40 dark:text-green-200'
    case 'warning':
      return 'bg-yellow-100 text-yellow-700 dark:bg-yellow-900/40 dark:text-yellow-200'
    case 'error':
      return 'bg-red-100 text-red-700 dark:bg-red-900/40 dark:text-red-200'
    default:
      return 'bg-blue-100 text-blue-700 dark:bg-blue-900/40 dark:text-blue-200'
  }
})

const statusText = computed(() => {
  if (props.active) {
    return props.positiveLabel ?? '已完成'
  }
  return props.negativeLabel ?? '未完成'
})
</script>
