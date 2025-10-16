import { createApp } from 'vue'
import { createPinia } from 'pinia'
import App from './App.vue'
import './style.css'
import { ensureOfflineLicense } from '@/api/offline'
import { useThemeStore } from '@/stores/theme'

const bootstrap = async () => {
  const licenseOk = await ensureOfflineLicense()
  if (!licenseOk) {
    return
  }

  const app = createApp(App)
  const pinia = createPinia()
  app.use(pinia)

  const themeStore = useThemeStore(pinia)
  await themeStore.initialize()

  app.mount('#app')
}

void bootstrap()
