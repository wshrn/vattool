import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'

export default defineConfig({
  plugins: [vue()],
  resolve: {
    alias: {
      '@': '/src'
    },
    conditions: ['tauri']
  },
  optimizeDeps: {
    exclude: ['@tauri-apps/api']
  },
  build: {
    target: 'es2021'
  },
  server: {
    port: 5173,
    strictPort: true
  }
})
