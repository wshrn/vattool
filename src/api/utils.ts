export const isTauriAvailable = (): boolean => {
  return typeof window !== 'undefined' && Boolean((window as any).__TAURI__)
}
