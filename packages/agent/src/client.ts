import type { AgentChunk, AgentExecuteRequest, ChatRoom } from './types'

export type TachyonAgentClientConfig = {
  /** Base URL of the API server, e.g. "https://api.n1.tachy.one" */
  apiBaseUrl: string
  /** Bearer token for authentication */
  accessToken: string
  /** Operator tenant ID (x-operator-id header) */
  tenantId?: string
  /** Optional user ID header */
  userId?: string
}

export class TachyonAgentError extends Error {
  constructor(
    message: string,
    public readonly status?: number,
    public readonly body?: string,
  ) {
    super(message)
    this.name = 'TachyonAgentError'
  }
}

export class TachyonAgentClient {
  private config: TachyonAgentClientConfig

  constructor(config: TachyonAgentClientConfig) {
    this.config = config
  }

  // ── Internal helpers ──────────────────────────────────────────────

  private getHeaders(): Record<string, string> {
    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
      Authorization: `Bearer ${this.config.accessToken}`,
    }
    if (this.config.tenantId) {
      headers['x-operator-id'] = this.config.tenantId
    }
    if (this.config.userId) {
      headers['x-user-id'] = this.config.userId
    }
    return headers
  }

  private buildUrl(path: string): string {
    const base = this.config.apiBaseUrl.replace(/\/+$/, '')
    return `${base}${path}`
  }

  private async request<T>(path: string, options?: RequestInit): Promise<T> {
    const url = this.buildUrl(path)
    const response = await fetch(url, {
      ...options,
      headers: {
        ...this.getHeaders(),
        ...options?.headers,
      },
    })
    if (!response.ok) {
      const body = await response.text().catch(() => '')
      throw new TachyonAgentError(
        `TachyonAgentClient request failed: ${response.status} ${response.statusText}${body ? ` - ${body}` : ''}`,
        response.status,
        body,
      )
    }
    return (await response.json()) as T
  }

  // ── Chatroom API ──────────────────────────────────────────────────

  async createChatRoom(title?: string): Promise<ChatRoom> {
    const data = await this.request<{ chatroom: ChatRoom }>(
      '/v1/llms/chatrooms',
      {
        method: 'POST',
        body: JSON.stringify({
          ...(title && { name: title }),
        }),
      },
    )
    return data.chatroom
  }

  // ── Agent API ─────────────────────────────────────────────────────

  async *stream(
    chatroomId: string,
    request: AgentExecuteRequest,
  ): AsyncGenerator<AgentChunk> {
    const url = this.buildUrl(
      `/v1/llms/sessions/${chatroomId}/agent/execute`,
    )
    const response = await fetch(url, {
      method: 'POST',
      headers: {
        ...this.getHeaders(),
        Accept: 'text/event-stream',
      },
      body: JSON.stringify(request),
    })
    if (!response.ok) {
      const body = await response.text().catch(() => '')
      throw new TachyonAgentError(
        `Agent execute failed: ${response.status}${body ? ` - ${body}` : ''}`,
        response.status,
        body,
      )
    }
    yield* readSSEStream(response)
  }

  async execute(
    chatroomId: string,
    request: AgentExecuteRequest,
  ): Promise<AgentChunk[]> {
    const chunks: AgentChunk[] = []
    for await (const chunk of this.stream(chatroomId, request)) {
      chunks.push(chunk)
    }
    return chunks
  }

  async getMessages(chatroomId: string): Promise<AgentChunk[]> {
    const data = await this.request<{ messages: AgentChunk[] }>(
      `/v1/llms/sessions/${chatroomId}/agent/messages`,
    )
    return data.messages
  }
}

// ── Private SSE reader ─────────────────────────────────────────────

async function* readSSEStream(response: Response): AsyncGenerator<AgentChunk> {
  const body = response.body
  if (!body) {
    throw new TachyonAgentError('Response body is null')
  }

  const reader = body.getReader()
  const decoder = new TextDecoder()
  let buffer = ''

  try {
    while (true) {
      const { done, value } = await reader.read()
      if (done) break

      buffer += decoder.decode(value, { stream: true })

      const lines = buffer.split('\n')
      // Keep the last incomplete line in the buffer
      buffer = lines.pop() ?? ''

      for (const line of lines) {
        const trimmed = line.trim()
        if (!trimmed || trimmed.startsWith(':')) {
          // Empty line or SSE comment, skip
          continue
        }
        if (trimmed.startsWith('data:')) {
          const data = trimmed.slice(5).trim()
          if (data === '[DONE]') {
            return
          }
          try {
            const chunk = JSON.parse(data) as AgentChunk
            yield chunk
          } catch {
            // Skip malformed JSON lines
          }
        }
      }
    }

    // Process any remaining data in the buffer
    if (buffer.trim()) {
      const trimmed = buffer.trim()
      if (trimmed.startsWith('data:')) {
        const data = trimmed.slice(5).trim()
        if (data !== '[DONE]') {
          try {
            const chunk = JSON.parse(data) as AgentChunk
            yield chunk
          } catch {
            // Skip malformed JSON
          }
        }
      }
    }
  } finally {
    reader.releaseLock()
  }
}
