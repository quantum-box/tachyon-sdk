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
