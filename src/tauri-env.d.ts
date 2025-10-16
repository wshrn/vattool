declare global {
  interface Window {
    __TAURI__?: {
      dialog?: {
        message?: (message: string, options?: { title?: string; type?: 'info' | 'warning' | 'error' }) => Promise<void>
      }
      app?: {
        exit?: (code?: number) => Promise<void>
      }
    }
  }
}

export {}
