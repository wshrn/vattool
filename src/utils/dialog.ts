import { isTauri } from './runtime'

type DialogType = 'info' | 'error' | 'warning' | 'success'

export interface DialogOptions {
  title: string
  type: DialogType
}

export const showDialogMessage = async (content: string, options: DialogOptions): Promise<void> => {
  if (typeof window === 'undefined') {
    return
  }

  if (!isTauri()) {
    window.alert(`${options.title}: ${content}`)
    return
  }

  const dialog = (window as any).__TAURI__?.dialog
  if (dialog?.message) {
    await dialog.message(content, options)
    return
  }

  window.alert(`${options.title}: ${content}`)
}
