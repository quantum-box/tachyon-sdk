import { TachyonAgentError } from './errors'

/**
 * One event of an agent session stream. `type` names the event, e.g.
 * `say`, `thinking`, `tool_call`, `tool_call_args`, `tool_call_pending`,
 * `tool_result`, `attempt_completion`, `error` or `done`.
 */
export type AgentSessionEvent = {
  type: string
  text?: string
  result?: string
  index?: number
  tool_id?: string
  tool_name?: string
  args?: unknown
  [key: string]: unknown
}

export type ReadAgentEventsOptions = {
  signal?: AbortSignal
  /** Fail when one event grows beyond this many characters. */
  maxEventChars?: number
}

const DEFAULT_MAX_EVENT_CHARS = 1_000_000

function parseRecord(record: string): AgentSessionEvent | null {
  const data = record
    .split('\n')
    .filter(line => line.startsWith('data:'))
    .map(line => line.slice(5).replace(/^ /, ''))
    .join('\n')
  if (!data.trim()) return null
  if (data.trim() === '[DONE]') return { type: 'done' }
  let value: unknown
  try {
    value = JSON.parse(data)
  } catch {
    throw new TachyonAgentError('Agent stream sent an event that is not JSON')
  }
  if (!value || typeof value !== 'object') {
    throw new TachyonAgentError('Agent stream sent an event without a type')
  }
  const event = value as { type?: unknown; data?: unknown }
  // Some servers wrap the payload: { type, data: { ... } }.
  const merged =
    event.data && typeof event.data === 'object'
      ? { ...(event.data as object), type: event.type }
      : event
  if (typeof (merged as { type?: unknown }).type !== 'string') {
    throw new TachyonAgentError('Agent stream sent an event without a type')
  }
  return merged as AgentSessionEvent
}

/**
 * Read a server-sent event stream from an agent session until `done`.
 * Throws when the stream ends before `done`, so a dropped connection is not
 * mistaken for a finished answer.
 */
export async function* readAgentEvents(
  body: ReadableStream<Uint8Array>,
  options: ReadAgentEventsOptions = {},
): AsyncGenerator<AgentSessionEvent> {
  const { signal } = options
  const maxChars = options.maxEventChars ?? DEFAULT_MAX_EVENT_CHARS
  const reader = body.getReader()
  const cancel = () => {
    void reader.cancel().catch(() => undefined)
  }
  signal?.addEventListener('abort', cancel, { once: true })
  const decoder = new TextDecoder()
  let buffer = ''
  const throwIfAborted = () => {
    if (signal?.aborted) {
      throw signal.reason ?? new TachyonAgentError('Agent stream was aborted')
    }
  }
  try {
    while (true) {
      throwIfAborted()
      const { value, done } = await reader.read()
      throwIfAborted()
      if (done) break
      buffer += decoder.decode(value, { stream: true }).replace(/\r/g, '')
      if (buffer.length > maxChars) {
        throw new TachyonAgentError('Agent stream event is too large')
      }
      let boundary = buffer.indexOf('\n\n')
      while (boundary !== -1) {
        const event = parseRecord(buffer.slice(0, boundary))
        buffer = buffer.slice(boundary + 2)
        if (event) {
          yield event
          if (event.type === 'done') return
        }
        boundary = buffer.indexOf('\n\n')
      }
    }
    buffer += decoder.decode()
    const event = parseRecord(buffer)
    if (event) {
      yield event
      if (event.type === 'done') return
    }
    throw new TachyonAgentError('Agent stream ended before completion')
  } finally {
    signal?.removeEventListener('abort', cancel)
    await reader.cancel().catch(() => undefined)
    reader.releaseLock()
  }
}
