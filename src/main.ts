import { createApp } from 'vue'
import { createPinia } from 'pinia'
import App from './App.vue'
import router from './router'
import { useThemeStore } from '@/stores/theme'
import { ToolboxAPI, isTauriAvailable } from '@/api'
import './assets/style.css'

const ensureOfflineLicense = async (): Promise<boolean> => {
  if (!isTauriAvailable()) {
    return true
  }

  try {
    const result = await ToolboxAPI.validateOfflineKey()
    if (!result.isValid) {
      const reason = result.reason ?? '离线密钥无效'
      const expiresAt = result.expiresAt ? `\n到期时间：${result.expiresAt}` : ''
      console.error(`离线密钥校验失败：${reason}${expiresAt}`)
      const { message } = await import('@tauri-apps/api/dialog')
      const { exit } = await import('@tauri-apps/api/process')
      await message('认证失败', { title: '认证失败', type: 'error' })
      await exit(0)
      return false
    }
    return true
  } catch (error) {
    const detail = error instanceof Error ? error.message : String(error)
    console.error(`离线密钥校验失败：${detail}`)
    const { message } = await import('@tauri-apps/api/dialog')
    const { exit } = await import('@tauri-apps/api/process')
    await message('认证失败', { title: '认证失败', type: 'error' })
    await exit(0)
    return false
  }
}

const bootstrap = async () => {
  const pass = await ensureOfflineLicense()
  if (!pass) {
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
