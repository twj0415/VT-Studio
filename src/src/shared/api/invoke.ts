import type { TauriCommandName } from './commands'

export type ApiAdapterKind = 'tauri'

type TauriCoreBridge = {
  invoke?: <TResponse>(command: string, payload?: unknown) => Promise<TResponse>
}

type TauriWindow = Window & {
  __TAURI__?: {
    core?: TauriCoreBridge
  }
}

function getTauriInvoke() {
  if (typeof window === 'undefined') return undefined
  return (window as TauriWindow).__TAURI__?.core?.invoke
}

export function getApiAdapter(): ApiAdapterKind {
  return 'tauri'
}

export async function invokeCommand<TResponse>(command: TauriCommandName | string, payload?: unknown): Promise<TResponse> {
  const tauriInvoke = getTauriInvoke()
  if (!tauriInvoke) throw new Error('Tauri command bridge is not connected. Run the desktop app to use real project data.')
  return tauriInvoke<TResponse>(command, payload)
}
