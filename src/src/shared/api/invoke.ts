import type { TauriCommandName } from './commands'

export type ApiAdapterKind = 'mock' | 'tauri'
export type ApiAdapterOverride = ApiAdapterKind | 'auto'
export type MockCommandHandler<TPayload = unknown, TResponse = unknown> = (payload: TPayload | undefined) => TResponse | Promise<TResponse>

type TauriCoreBridge = {
  invoke?: <TResponse>(command: string, payload?: unknown) => Promise<TResponse>
}

type TauriWindow = Window & {
  __TAURI__?: {
    core?: TauriCoreBridge
  }
}

let activeAdapter: ApiAdapterOverride = 'auto'
const mockHandlers = new Map<string, MockCommandHandler>()

function getTauriInvoke() {
  if (typeof window === 'undefined') return undefined
  return (window as TauriWindow).__TAURI__?.core?.invoke
}

function resolveApiAdapter(): ApiAdapterKind {
  if (activeAdapter !== 'auto') return activeAdapter
  return getTauriInvoke() ? 'tauri' : 'mock'
}

export function getApiAdapter(): ApiAdapterKind {
  return resolveApiAdapter()
}

export function getApiAdapterOverride() {
  return activeAdapter
}

export function setApiAdapter(adapter: ApiAdapterOverride) {
  activeAdapter = adapter
}

export function registerMockCommand<TPayload = unknown, TResponse = unknown>(command: TauriCommandName | string, handler: MockCommandHandler<TPayload, TResponse>) {
  mockHandlers.set(command, handler as MockCommandHandler)
}

export function clearMockCommands() {
  mockHandlers.clear()
}

export async function invokeCommand<TResponse>(command: TauriCommandName | string, payload?: unknown): Promise<TResponse> {
  const adapter = resolveApiAdapter()

  if (adapter === 'mock') {
    const handler = mockHandlers.get(command)
    if (!handler) throw new Error(`Mock command is not registered: ${command}`)
    return (await handler(payload)) as TResponse
  }

  const tauriInvoke = getTauriInvoke()
  if (!tauriInvoke) throw new Error('Tauri command bridge is not connected yet.')
  return tauriInvoke<TResponse>(command, payload)
}
