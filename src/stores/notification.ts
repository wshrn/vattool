import { defineStore } from 'pinia'

export type NotificationType = 'success' | 'warning' | 'error' | 'info'

export interface NotificationItem {
  id: number
  title: string
  message: string
  type: NotificationType
}

let counter = 0

export const useNotificationStore = defineStore('notification', {
  state: () => ({
    items: [] as NotificationItem[]
  }),
  actions: {
    push(item: Omit<NotificationItem, 'id'>, duration = 4000) {
      const id = ++counter
      this.items.push({ ...item, id })
      if (duration > 0) {
        window.setTimeout(() => {
          this.remove(id)
        }, duration)
      }
    },
    remove(id: number) {
      this.items = this.items.filter((item) => item.id !== id)
    }
  }
})
