import { TachyonAgentError } from './errors'
import { type AgentSessionEvent, readAgentEvents } from './events'

/**
 * How the session client reaches the server. Apps that already have an API
 * client (auth, tenant headers, timeouts, a proxy in front of Tachyon) pass
 * their own; otherwise use `fetchAgentTransport`.
 */
export type AgentSessionTransport = {
  /** POST JSON and return the parsed JSON body, or undefined when empty. */
  postJson(path: string, body: unknown, signal?: AbortSignal): Promise<unknown>
  /** POST JSON and return the response body as a server-sent event stream. */
  postStream(
    path: string,
    body: unknown,
    signal?: AbortSignal,
  ): Promise<ReadableStream<Uint8Array>>
}

/** Where each call goes. Defaults are Tachyon's own agent session API. */
export type AgentSessionPaths = {
  sessions: string
  execute(sessionId: string): string
  toolResult(sessionId: string): string
}

export const TACHYON_AGENT_SESSION_PATHS: AgentSessionPaths = {
  sessions: '/v1/llms/sessions',
  execute: id => `/v1/llms/sessions/${encodeURIComponent(id)}/agent/execute`,
  toolResult: id =>
    `/v1/llms/sessions/${encodeURIComponent(id)}/agent/tool-result`,
}

export type AgentSessionAttachment = {
  filename: string
  content_type: string
  /** Base64 file contents. */
  data: string
}

export type AgentSessionExecuteRequest = {
  task: string
  attachments?: AgentSessionAttachment[]
  /** Server-specific fields, passed through unchanged. */
  [key: string]: unknown
}

export type AgentToolResult = {
  tool_id: string
  /** Text the model reads; see `toolResultText` for a size-bounded form. */
  result: string
  is_error: boolean
}

export type CreateAgentSessionRequest = {
  name?: string
  metadata?: Record<string, unknown>
  [key: string]: unknown
}

export type TachyonAgentSessionClientConfig = {
  transport: AgentSessionTransport
  paths?: Partial<AgentSessionPaths>
  /** Fail when one stream event grows beyond this many characters. */
  maxEventChars?: number
}

const SESSION_ID = /^as_[\w-]+$/

/**
 * Client for Tachyon agent sessions: the server runs the agent loop and keeps
 * history; the app streams events and answers client-side tool calls.
 */
export class TachyonAgentSessionClient {
  private readonly transport: AgentSessionTransport
  private readonly paths: AgentSessionPaths
  private readonly maxEventChars?: number

  constructor(config: TachyonAgentSessionClientConfig) {
    this.transport = config.transport
    this.paths = { ...TACHYON_AGENT_SESSION_PATHS, ...config.paths }
    this.maxEventChars = config.maxEventChars
  }

  /** Start a session and return its id (`as_...`). */
  async createSession(
    request?: CreateAgentSessionRequest,
    signal?: AbortSignal,
  ): Promise<string> {
    const value = await this.transport.postJson(
      this.paths.sessions,
      request,
      signal,
    )
    const id = (value as { session_id?: unknown } | undefined)?.session_id
    if (typeof id !== 'string' || !SESSION_ID.test(id)) {
      throw new TachyonAgentError('Agent session response has no session id')
    }
    return id
  }

  /** Run one task and stream its events until `done`. */
  async *execute(
    sessionId: string,
    request: AgentSessionExecuteRequest,
    signal?: AbortSignal,
  ): AsyncGenerator<AgentSessionEvent> {
    const body = await this.transport.postStream(
      this.paths.execute(sessionId),
      request,
      signal,
    )
    yield* readAgentEvents(body, { signal, maxEventChars: this.maxEventChars })
  }

  /** Answer a `tool_call_pending` event the app ran itself. */
  async submitToolResult(
    sessionId: string,
    result: AgentToolResult,
    signal?: AbortSignal,
  ): Promise<void> {
    await this.transport.postJson(
      this.paths.toolResult(sessionId),
      result,
      signal,
    )
  }
}

/**
 * Serialize a tool result for the model, replacing it with a truncated
 * preview when it exceeds `budget` characters.
 */
export function toolResultText(
  result: unknown,
  budget: number,
  instruction = 'Use pagination; this is incomplete.',
): string {
  const text = typeof result === 'string' ? result : JSON.stringify(result)
  if (text.length <= budget) return text
  return JSON.stringify({
    truncated: true,
    preview: text.slice(0, Math.max(0, budget - 1000)),
    instruction,
  })
}

export type FetchAgentTransportConfig = {
  /** Base URL of the API server, e.g. "https://api.n1.tachy.one" */
  apiBaseUrl: string
  /** Bearer token, or a function returning a fresh one per request. */
  accessToken: string | (() => string | Promise<string>)
  /** Operator tenant ID (x-operator-id header) */
  tenantId?: string
  /** Platform ID (x-platform-id header) */
  platformId?: string
  headers?: Record<string, string>
  fetch?: typeof fetch
}

/** Transport that calls the API directly with `fetch`. */
export function fetchAgentTransport(
  config: FetchAgentTransportConfig,
): AgentSessionTransport {
  const send = async (
    path: string,
    body: unknown,
    accept: string,
    signal?: AbortSignal,
  ) => {
    const token =
      typeof config.accessToken === 'function'
        ? await config.accessToken()
        : config.accessToken
    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
      Accept: accept,
      Authorization: `Bearer ${token}`,
      ...config.headers,
    }
    if (config.tenantId) headers['x-operator-id'] = config.tenantId
    if (config.platformId) headers['x-platform-id'] = config.platformId
    const response = await (config.fetch ?? fetch)(
      `${config.apiBaseUrl.replace(/\/+$/, '')}${path}`,
      {
        method: 'POST',
        headers,
        body: body === undefined ? undefined : JSON.stringify(body),
        signal,
      },
    )
    if (!response.ok) {
      const text = await response.text().catch(() => '')
      throw new TachyonAgentError(
        `Agent request failed: ${response.status}${text ? ` - ${text}` : ''}`,
        response.status,
        text,
      )
    }
    return response
  }
  return {
    async postJson(path, body, signal) {
      const response = await send(path, body, 'application/json', signal)
      const text = await response.text()
      return text ? JSON.parse(text) : undefined
    },
    async postStream(path, body, signal) {
      const response = await send(path, body, 'text/event-stream', signal)
      if (!response.body) {
        throw new TachyonAgentError('Agent stream response has no body')
      }
      return response.body
    },
  }
}
