import { describe, expect, it } from 'vitest'
import { readAgentEvents } from './events'
import {
  type AgentSessionTransport,
  TachyonAgentSessionClient,
  fetchAgentTransport,
  toolResultText,
} from './session'

function stream(...chunks: string[]) {
  const encoder = new TextEncoder()
  return new ReadableStream<Uint8Array>({
    start(controller) {
      for (const chunk of chunks) controller.enqueue(encoder.encode(chunk))
      controller.close()
    },
  })
}

async function collect<T>(events: AsyncIterable<T>) {
  const out: T[] = []
  for await (const event of events) out.push(event)
  return out
}

describe('readAgentEvents', () => {
  it('reads events split across chunks and unwraps data envelopes', async () => {
    const events = await collect(
      readAgentEvents(
        stream(
          'data: {"type":"say","te',
          'xt":"hi"}\r\n\r\n',
          ': keep-alive\n\n',
          'data: {"type":"tool_call_pending","data":{"tool_id":"t1"}}\n\n',
          'data: [DONE]\n\n',
          'data: {"type":"say","text":"ignored"}\n\n',
        ),
      ),
    )
    expect(events).toEqual([
      { type: 'say', text: 'hi' },
      { type: 'tool_call_pending', tool_id: 't1' },
      { type: 'done' },
    ])
  })

  it('accepts a final event without a trailing blank line', async () => {
    const events = await collect(readAgentEvents(stream('data: [DONE]')))
    expect(events).toEqual([{ type: 'done' }])
  })

  it('fails when the stream ends before done', async () => {
    await expect(
      collect(readAgentEvents(stream('data: {"type":"say"}\n\n'))),
    ).rejects.toThrow('ended before completion')
  })

  it('fails on events that are too large or malformed', async () => {
    await expect(
      collect(readAgentEvents(stream('data: 1234567890'), { maxEventChars: 5 })),
    ).rejects.toThrow('too large')
    await expect(
      collect(readAgentEvents(stream('data: {oops\n\n'))),
    ).rejects.toThrow('not JSON')
    await expect(
      collect(readAgentEvents(stream('data: {"text":"x"}\n\n'))),
    ).rejects.toThrow('without a type')
  })

  it('stops when aborted', async () => {
    const controller = new AbortController()
    controller.abort(new Error('stopped'))
    await expect(
      collect(
        readAgentEvents(stream('data: [DONE]\n\n'), {
          signal: controller.signal,
        }),
      ),
    ).rejects.toThrow('stopped')
  })
})

describe('TachyonAgentSessionClient', () => {
  function recorder(json: unknown, body = 'data: [DONE]\n\n') {
    const calls: { kind: string; path: string; body: unknown }[] = []
    const transport: AgentSessionTransport = {
      async postJson(path, value) {
        calls.push({ kind: 'json', path, body: value })
        return json
      },
      async postStream(path, value) {
        calls.push({ kind: 'stream', path, body: value })
        return stream(body)
      },
    }
    return { calls, transport }
  }

  it('uses Tachyon session paths by default', async () => {
    const { calls, transport } = recorder({ session_id: 'as_1' })
    const client = new TachyonAgentSessionClient({ transport })
    expect(await client.createSession({ name: 'Agent' })).toBe('as_1')
    expect(await collect(client.execute('as_1', { task: 'hi' }))).toEqual([
      { type: 'done' },
    ])
    await client.submitToolResult('as_1', {
      tool_id: 't1',
      result: '{}',
      is_error: false,
    })
    expect(calls).toEqual([
      { kind: 'json', path: '/v1/llms/sessions', body: { name: 'Agent' } },
      {
        kind: 'stream',
        path: '/v1/llms/sessions/as_1/agent/execute',
        body: { task: 'hi' },
      },
      {
        kind: 'json',
        path: '/v1/llms/sessions/as_1/agent/tool-result',
        body: { tool_id: 't1', result: '{}', is_error: false },
      },
    ])
  })

  it('goes through an app proxy when paths are given', async () => {
    const { calls, transport } = recorder({ session_id: 'as_2' })
    const client = new TachyonAgentSessionClient({
      transport,
      paths: {
        sessions: '/v1/app/agent/sessions',
        execute: id => `/v1/app/agent/sessions/${id}/execute`,
      },
    })
    await client.createSession()
    await collect(client.execute('as_2', { task: 'hi', mode: 'x' }))
    expect(calls.map(call => call.path)).toEqual([
      '/v1/app/agent/sessions',
      '/v1/app/agent/sessions/as_2/execute',
    ])
    expect(calls[1].body).toEqual({ task: 'hi', mode: 'x' })
  })

  it('rejects a session response without a valid id', async () => {
    for (const value of [undefined, {}, { session_id: 'bad' }]) {
      const client = new TachyonAgentSessionClient({
        transport: recorder(value).transport,
      })
      await expect(client.createSession()).rejects.toThrow('no session id')
    }
  })
})

describe('toolResultText', () => {
  it('keeps small results and truncates large ones', () => {
    expect(toolResultText({ ok: true }, 100)).toBe('{"ok":true}')
    const text = toolResultText('x'.repeat(5000), 2000)
    expect(JSON.parse(text)).toEqual({
      truncated: true,
      preview: 'x'.repeat(1000),
      instruction: 'Use pagination; this is incomplete.',
    })
  })
})

describe('fetchAgentTransport', () => {
  it('sends auth and tenant headers and reports failures', async () => {
    const seen: { url: string; init: RequestInit }[] = []
    const transport = fetchAgentTransport({
      apiBaseUrl: 'https://api.example/',
      accessToken: async () => 'token',
      tenantId: 'tn_1',
      platformId: 'tn_p',
      fetch: async (url, init) => {
        seen.push({ url: String(url), init: init ?? {} })
        return String(url).endsWith('/fail')
          ? new Response('nope', { status: 403 })
          : new Response('{"session_id":"as_1"}')
      },
    })
    expect(await transport.postJson('/v1/llms/sessions', { a: 1 })).toEqual({
      session_id: 'as_1',
    })
    expect(seen[0].url).toBe('https://api.example/v1/llms/sessions')
    expect(seen[0].init.headers).toMatchObject({
      Authorization: 'Bearer token',
      'x-operator-id': 'tn_1',
      'x-platform-id': 'tn_p',
    })
    await expect(transport.postJson('/fail', undefined)).rejects.toMatchObject({
      status: 403,
    })
  })
})
