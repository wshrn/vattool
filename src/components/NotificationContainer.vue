<template>
  <transition-group name="list" tag="div" class="fixed right-6 top-14 space-y-3 z-50">
    <div
      v-for="item in notifications"
      :key="item.id"
      :class="['notification', item.type]"
      class="w-80"
    >
      <div class="flex items-start space-x-3">
        <div class="flex-1">
          <h3 class="text-sm font-semibold">{{ item.title }}</h3>
          <p class="text-xs mt-1 leading-relaxed whitespace-pre-line">{{ item.message }}</p>
        </div>
        <button class="btn-ghost px-2 py-1" @click="dismiss(item.id)">关闭</button>
      </div>
    </div>
  </transition-group>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useNotificationStore } from '@/stores/notification'

const notificationStore = useNotificationStore()
const notifications = computed(() => notificationStore.items)

const dismiss = (id: number) => {
  notificationStore.remove(id)
}
</script>
