# @tachyon-sdk/agent

TypeScript client SDK for the [Tachyon Agent API](https://api.n1.tachy.one).

## Installation

```bash
npm install @tachyon-sdk/agent
```

## Quickstart

```typescript
import { TachyonAgentClient } from '@tachyon-sdk/agent'

const client = new TachyonAgentClient({
  apiKey: process.env.TACHYON_API_KEY!,
  baseUrl: 'https://api.n1.tachy.one',
})

for await (const event of client.streamAgent('Slackに本番デプロイ完了を通知して')) {
  if (event.type === 'text') {
    process.stdout.write(event.content)
  } else if (event.type === 'tool_call') {
    console.log('[tool]', event.content)
  } else if (event.type === 'done') {
    break
  }
}
```

## Functional API

```typescript
import { streamAgent } from '@tachyon-sdk/agent'

for await (const event of streamAgent('Hello!', {
  apiKey: process.env.TACHYON_API_KEY!,
  baseUrl: 'https://api.n1.tachy.one',
})) {
  if (event.type === 'text') process.stdout.write(event.content)
}
```

## Agent sessions (client-side tools)

`TachyonAgentSessionClient` talks to the session API
(`/v1/llms/sessions/...`). Tachyon runs the agent loop and keeps history; the
app streams events and answers `tool_call_pending` with tools it runs itself.

```typescript
import {
  TachyonAgentSessionClient,
  fetchAgentTransport,
  toolResultText,
} from '@tachyon-sdk/agent'

const agent = new TachyonAgentSessionClient({
  transport: fetchAgentTransport({
    apiBaseUrl: 'https://api.n1.tachy.one',
    accessToken: () => getAccessToken(),
    tenantId: 'tn_...',
  }),
})

const session = await agent.createSession({ name: 'My Agent' })
for await (const event of agent.execute(session, { task: 'Hello!' })) {
  if (event.type === 'say') console.log(event.text)
  if (event.type === 'tool_call_pending' && event.tool_id) {
    const result = await runTool(event.tool_name, event.args)
    await agent.submitToolResult(session, {
      tool_id: event.tool_id,
      result: toolResultText(result, 16_000),
      is_error: false,
    })
  }
}
```

An app that proxies the agent through its own server passes its own
`transport` (reusing its API client's auth and timeouts) and `paths`:

```typescript
new TachyonAgentSessionClient({
  transport: myTransport,
  paths: {
    sessions: '/v1/my-app/agent/sessions',
    execute: id => `/v1/my-app/agent/sessions/${id}/execute`,
    toolResult: id => `/v1/my-app/agent/sessions/${id}/tool-result`,
  },
})
```

`readAgentEvents(body)` reads a session event stream on its own. It throws
when the stream ends before `done`.

## React (SSE streaming)

```tsx
import { useState } from 'react'
import { streamAgent } from '@tachyon-sdk/agent'

export function AgentChat() {
  const [output, setOutput] = useState('')

  const run = async () => {
    setOutput('')
    for await (const event of streamAgent('こんにちは', {
      apiKey: import.meta.env.VITE_TACHYON_API_KEY,
      baseUrl: 'https://api.n1.tachy.one',
    })) {
      if (event.type === 'text') setOutput((s) => s + event.content)
    }
  }

  return (
    <div>
      <button onClick={run}>Run Agent</button>
      <pre>{output}</pre>
    </div>
  )
}
```

## Environment Variables

| Variable | Description |
|----------|-------------|
| `TACHYON_API_KEY` | API key issued from the Tachyon dashboard |
| `TACHYON_API_URL` | Base URL (default: `https://api.n1.tachy.one`) |

## Types

```typescript
type AgentEvent = {
  type: 'text' | 'tool_call' | 'done'
  content: string
}

type TachyonAgentClientConfig = {
  apiKey: string   // Bearer token
  baseUrl: string  // e.g. https://api.n1.tachy.one
}
```

## Examples

- [`examples/node-cli`](../../examples/node-cli) — Node.js streaming CLI
- [`examples/react-agent-chat`](../../examples/react-agent-chat) — React chat UI with SSE streaming
