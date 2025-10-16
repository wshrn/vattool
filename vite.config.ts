import { fileURLToPath, URL } from 'node:url'
import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'

export default defineConfig({
  plugins: [vue()],
  resolve: {
    alias: {
      '@': fileURLToPath(new URL('./src', import.meta.url))
    }
  },
  server: {
    port: 1420,
    strictPort: true
  },
  optimizeDeps: {
    exclude: ['@tauri-apps/api']
  },
  build: {
    rollupOptions: {
      external: [/^@tauri-apps\/api\//, '@tauri-apps/api']
    }
  }
})
