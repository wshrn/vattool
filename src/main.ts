import { createApp } from 'vue'
import { createPinia } from 'pinia'
import App from './App.vue'
import './style.css'
import { useThemeStore } from './stores/theme'
import { ensureOfflineLicense } from './utils/license'

const bootstrap = async () => {
  await ensureOfflineLicense()

  const app = createApp(App)
  const pinia = createPinia()
  app.use(pinia)

  const themeStore = useThemeStore(pinia)
  void themeStore.initialize()

  app.mount('#app')
}

void bootstrap()
