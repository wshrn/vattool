import { createApp } from 'vue'
import { createPinia } from 'pinia'
import App from './App.vue'
import './style.css'
import { useThemeStore } from './stores/theme'
import { ensureOfflineLicense } from './api/offline'

const bootstrap = async () => {
  const app = createApp(App)
  const pinia = createPinia()
  app.use(pinia)

  const themeStore = useThemeStore(pinia)
  await themeStore.initialize()

  const licenseOk = await ensureOfflineLicense()
  if (!licenseOk) {
    return
  }

  app.mount('#app')
}

void bootstrap()
