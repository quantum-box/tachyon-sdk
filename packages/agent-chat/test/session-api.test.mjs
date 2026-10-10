import assert from 'node:assert/strict'
import { afterEach, test } from 'node:test'
import { AgentChatClient } from '../src/client/AgentChatClient.ts'

const originalFetch = globalThis.fetch
afterEach(() => { globalThis.fetch = originalFetch })
const client = () => new AgentChatClient({
  apiBaseUrl: 'https://api.example.test/',
  accessToken: 'fixture-token',
  tenantId: 'tn_fixture',
})

test('a new conversation is a Session and execution keeps its identity', async () => {
  const calls = []
  globalThis.fetch = async (url, options) => {
    calls.push({ url, options })
    if (calls.length === 1) return Response.json({ session_id: 'as_fixture', created_at: '2026-10-10T00:00:00Z' })
    return new Response('data: {"type":"done"}\n\n', { headers: { 'Content-Type': 'text/event-stream' } })
  }
  const api = client()
  const session = await api.createSession('Named conversation')
  await api.executeAgent(session.session_id, { task: 'hello' })
  assert.equal(calls[0].url, 'https://api.example.test/v1/llms/sessions')
  assert.deepEqual(JSON.parse(calls[0].options.body), { name: 'Named conversation' })
  assert.equal(calls[1].url, 'https://api.example.test/v1/llms/sessions/as_fixture/agent/execute')
  for (const { options } of calls) {
    assert.equal(options.headers.Authorization, 'Bearer fixture-token')
    assert.equal(options.headers['x-operator-id'], 'tn_fixture')
  }
})

test('listing and deletion use Session endpoints, including empty delete responses', async () => {
  const calls = []
  globalThis.fetch = async (url, options) => {
    calls.push({ url, options })
    return options?.method === 'DELETE'
      ? new Response(null, { status: 204 })
      : Response.json({ sessions: [{ id: 'as_fixture' }] })
  }
  const api = client()
  assert.deepEqual(await api.getSessions(10, 20), [{ id: 'as_fixture' }])
  await api.deleteSession('as_fixture')
  assert.equal(calls[0].url, 'https://api.example.test/v1/llms/sessions?limit=10&offset=20')
  assert.equal(calls[1].url, 'https://api.example.test/v1/llms/sessions/as_fixture')
  assert.equal(calls[1].options.method, 'DELETE')
})

test('failed Session creation never falls back to creating a ChatRoom', async () => {
  const urls = []
  globalThis.fetch = async url => {
    urls.push(url)
    return new Response('denied', { status: 403 })
  }
  await assert.rejects(client().createSession(), /403.*denied/)
  assert.deepEqual(urls, ['https://api.example.test/v1/llms/sessions'])
})
