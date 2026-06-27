export const taskProgressEventName = 'task://progress'

export interface TaskProgressEventPayload {
  traceId: string
  projectId: string
  taskId: string
  taskStepId?: string
  stepKind?: string
  status: string
  progress: number
  message: string
  errorCode?: string
  itemId?: string
}

type TauriEventApi = {
  listen?: <TPayload>(event: string, handler: (event: { payload: TPayload }) => void) => Promise<() => void>
}

type TauriWindow = Window & {
  __TAURI__?: {
    event?: TauriEventApi
  }
}

function getTauriListen() {
  if (typeof window === 'undefined') return undefined
  return (window as TauriWindow).__TAURI__?.event?.listen
}

export async function listenTaskProgress(handler: (payload: TaskProgressEventPayload) => void): Promise<() => void> {
  const listen = getTauriListen()
  if (!listen) return () => {}

  return listen<TaskProgressEventPayload>(taskProgressEventName, (event) => {
    handler(event.payload)
  })
}
