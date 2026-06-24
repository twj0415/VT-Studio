import { normalizeApiError } from './errors'
import { invokeCommand } from './invoke'

export async function callCommand<TResponse, TPayload = unknown>(command: string, payload?: TPayload): Promise<TResponse> {
  try {
    return await invokeCommand<TResponse>(command, payload)
  } catch (error) {
    throw normalizeApiError(error)
  }
}
