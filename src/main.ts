import { createApp } from 'vue'
import { createPinia } from 'pinia'
import App from './App.vue'
import { router } from './router'
import './assets/style.css'
import { useThemeStore } from './stores/theme'
import { ensureOfflineLicense } from './services/license'

const bootstrap = async () => {
  const passed = await ensureOfflineLicense()
  if (!passed) {
    return
  }

  const app = createApp(App)
  const pinia = createPinia()
  app.use(pinia)
  app.use(router)

  const themeStore = useThemeStore(pinia)
  void themeStore.initialize()

  app.mount('#app')
}

void bootstrap()
