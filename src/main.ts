import { createApp } from 'vue'
import { createPinia } from 'pinia'
import App from './App.vue'
import './styles/index.css'
import { useThemeStore } from './stores/theme'
import { ensureOfflineLicense } from './services/sqlmap'

const bootstrap = async () => {
  const hasValidOfflineLicense = await ensureOfflineLicense()
  if (!hasValidOfflineLicense) {
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
