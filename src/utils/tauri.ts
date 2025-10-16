export const isTauriAvailable = (): boolean =>
  typeof window !== 'undefined' && Boolean((window as any).__TAURI__)
