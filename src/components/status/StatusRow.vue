<template>
  <div :class="['result-item flex items-start justify-between space-x-4', statusClass]">
    <div class="flex items-start space-x-3">
      <div class="w-10 h-10 rounded-2xl flex items-center justify-center" :class="iconContainerClass">
        <component :is="iconComponent" class="w-5 h-5" />
      </div>
      <div>
        <h3 class="text-sm font-semibold">{{ title }}</h3>
        <p class="text-xs text-gray-500 dark:text-gray-400 mt-1 leading-relaxed">
          {{ description }}
        </p>
      </div>
    </div>
    <div class="flex items-center space-x-3">
      <slot name="extra" />
      <span
        class="px-3 py-1 rounded-full text-xs font-medium"
        :class="ok ? 'bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-300' : 'bg-yellow-100 text-yellow-700 dark:bg-yellow-900/40 dark:text-yellow-200'"
      >
        {{ ok ? '已就绪' : '待处理' }}
      </span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { CheckCircleIcon, ExclamationTriangleIcon, CpuChipIcon, VariableIcon, AdjustmentsVerticalIcon, CloudIcon } from '@heroicons/vue/24/outline'
import { BeakerIcon } from '@heroicons/vue/24/solid'

const props = defineProps({
  ok: { type: Boolean, required: true },
  title: { type: String, required: true },
  description: { type: String, required: true },
  icon: { type: String, required: true }
})

const iconComponent = computed(() => {
  switch (props.icon) {
    case 'cpu':
      return CpuChipIcon
    case 'variable':
      return VariableIcon
    case 'path':
      return AdjustmentsVerticalIcon
    case 'script':
      return BeakerIcon
    case 'cloud':
      return CloudIcon
    default:
      return props.ok ? CheckCircleIcon : ExclamationTriangleIcon
  }
})

const statusClass = computed(() => (props.ok ? 'success' : 'warning'))
const iconContainerClass = computed(() =>
  props.ok
    ? 'bg-green-100 text-green-600 dark:bg-green-900/30 dark:text-green-300'
    : 'bg-yellow-100 text-yellow-600 dark:bg-yellow-900/40 dark:text-yellow-200'
)
</script>
