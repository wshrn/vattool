<template>
  <div
    class="result-item"
    :class="statusClass"
    role="status"
  >
    <div class="flex items-center justify-between">
      <div class="flex items-center space-x-3">
        <span
          class="w-2 h-2 rounded-full"
          :class="dotClass"
          aria-hidden="true"
        ></span>
        <span class="text-sm font-medium">{{ label }}</span>
      </div>
      <span class="text-sm text-gray-600 dark:text-gray-300">{{ message }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

type StatusType = 'success' | 'error' | 'warning' | 'info'

const props = defineProps<{ label: string; message: string; status: StatusType }>()

const statusClass = computed(() => {
  switch (props.status) {
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

const dotClass = computed(() => {
  switch (props.status) {
    case 'success':
      return 'bg-apple-green'
    case 'warning':
      return 'bg-apple-orange'
    case 'error':
      return 'bg-apple-red'
    default:
      return 'bg-apple-blue'
  }
})
</script>
