import { createApp } from 'vue'
import { createPinia } from 'pinia'
import App from './App.vue'
import './styles/index.css'
import { useThemeStore } from './stores/theme'
import { ensureOfflineLicense } from './services/sqlmap'

const bootstrap = async () => {
  const licensed = await ensureOfflineLicense()
  if (!licensed) {
    console.error('离线密钥认证失败，已终止应用启动流程')
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
