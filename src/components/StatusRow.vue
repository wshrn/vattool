<template>
  <div class="result-item" :class="statusClass">
    <div class="flex items-center justify-between">
      <div class="flex items-center gap-3">
        <component :is="icon" class="w-5 h-5" :class="iconClass" />
        <div>
          <p class="font-medium text-gray-800 dark:text-gray-100">{{ title }}</p>
          <p class="text-xs text-gray-500 dark:text-gray-400 mt-1">
            {{ description }}
          </p>
        </div>
      </div>
      <span class="text-sm font-medium" :class="badgeClass">{{ badgeText }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { CheckCircleIcon, ExclamationTriangleIcon, InformationCircleIcon } from '@heroicons/vue/24/solid'
import type { StatusItem } from '@/types/python'

const props = defineProps<{
  title: string
  item?: StatusItem | null
  successText: string
  failedText: string
}>()

const icon = computed(() => {
  if (!props.item) {
    return InformationCircleIcon
  }
  return props.item.status ? CheckCircleIcon : ExclamationTriangleIcon
})

const description = computed(() => {
  if (!props.item) {
    return '无法获取状态信息，请确保已在 Tauri 环境中运行。'
  }
  return props.item.status ? props.item.message || props.successText : props.item.message || props.failedText
})

const badgeText = computed(() => {
  if (!props.item) {
    return '未知'
  }
  return props.item.status ? '已完成' : '待处理'
})

const statusClass = computed(() => {
  if (!props.item) {
    return 'info'
  }
  return props.item.status ? 'success' : 'warning'
})

const iconClass = computed(() => {
  if (!props.item) {
    return 'text-blue-500'
  }
  return props.item.status ? 'text-green-500' : 'text-yellow-500'
})

const badgeClass = computed(() => {
  if (!props.item) {
    return 'text-blue-500'
  }
  return props.item.status ? 'text-green-500' : 'text-yellow-500'
})
</script>
