import type { TauriCommandName } from './commands'

export type ApiAdapterKind = 'mock' | 'tauri'
export type MockCommandHandler<TPayload = unknown, TResponse = unknown> = (payload: TPayload | undefined) => TResponse | Promise<TResponse>

type TauriCoreBridge = {
  invoke?: <TResponse>(command: string, payload?: unknown) => Promise<TResponse>
}

type TauriWindow = Window & {
  __TAURI__?: {
    core?: TauriCoreBridge
  }
}

let activeAdapter: ApiAdapterKind = 'mock'
const mockHandlers = new Map<string, MockCommandHandler>()

export function getApiAdapter() {
  return activeAdapter
}

export function setApiAdapter(adapter: ApiAdapterKind) {
  activeAdapter = adapter
}

export function registerMockCommand<TPayload = unknown, TResponse = unknown>(command: TauriCommandName | string, handler: MockCommandHandler<TPayload, TResponse>) {
  mockHandlers.set(command, handler as MockCommandHandler)
}

export function clearMockCommands() {
  mockHandlers.clear()
}

export async function invokeCommand<TResponse>(command: TauriCommandName | string, payload?: unknown): Promise<TResponse> {
  if (activeAdapter === 'mock') {
    const handler = mockHandlers.get(command)
    if (!handler) throw new Error(`Mock command is not registered: ${command}`)
    return (await handler(payload)) as TResponse
  }

  const tauriInvoke = (window as TauriWindow).__TAURI__?.core?.invoke
  if (!tauriInvoke) throw new Error('Tauri command bridge is not connected yet.')
  return tauriInvoke<TResponse>(command, payload)
}
