import { TachyonAgentClient } from './client'

export type StreamAgentOptions = {
  apiKey?: string
  accessToken?: string
  baseUrl?: string
  apiBaseUrl?: string
  tenantId?: string
  userId?: string
}

export type StreamEvent =
  | { type: 'text'; content: string }
  | { type: 'tool_call'; content: string }
  | { type: 'done' }

export async function* streamAgent(
  prompt: string,
  options: StreamAgentOptions,
): AsyncGenerator<StreamEvent> {
  const accessToken = options.accessToken ?? options.apiKey
  const apiBaseUrl = options.apiBaseUrl ?? options.baseUrl

  if (!accessToken) {
    throw new Error(
      'streamAgent: accessToken (or apiKey) is required',
    )
  }
  if (!apiBaseUrl) {
    throw new Error(
      'streamAgent: apiBaseUrl (or baseUrl) is required',
    )
  }

  const client = new TachyonAgentClient({
    apiBaseUrl,
    accessToken,
    tenantId: options.tenantId,
    userId: options.userId,
  })

  const chatroom = await client.createChatRoom()

  for await (const chunk of client.stream(chatroom.id, { task: prompt })) {
    if (chunk.type === 'say') {
      yield { type: 'text', content: chunk.text ?? '' }
    } else if (chunk.type === 'tool_call') {
      yield { type: 'tool_call', content: chunk.tool_name ?? '' }
    } else if (chunk.type === 'completion') {
      yield { type: 'done' }
    }
    // Other chunk types are skipped
  }
}
