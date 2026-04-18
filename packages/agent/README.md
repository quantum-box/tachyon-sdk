# @quantumbox/agent

TypeScript client SDK for the Tachyon Agent API.

## Installation

```bash
npm install @quantumbox/agent
```

## Usage

### TachyonAgentClient

```typescript
import { TachyonAgentClient } from '@quantumbox/agent'

const client = new TachyonAgentClient({
  apiKey: 'your-api-key',
  baseUrl: 'https://api.tachyon.ai',
})

for await (const event of client.streamAgent('Hello, agent!')) {
  if (event.type === 'text') {
    process.stdout.write(event.content)
  } else if (event.type === 'tool_call') {
    console.log('Tool call:', event.content)
  } else if (event.type === 'done') {
    break
  }
}
```

### streamAgent (functional API)

```typescript
import { streamAgent } from '@quantumbox/agent'

for await (const event of streamAgent('Hello!', { apiKey: '...', baseUrl: '...' })) {
  console.log(event)
}
```

## Types

```typescript
type AgentEvent = {
  type: 'text' | 'tool_call' | 'done'
  content: string
}
```

## Notes

> **v0.1 skeleton**: API endpoint paths (e.g. `/v1/agent/execute`) are illustrative.
> Exact endpoints will be finalized once the Tachyon Agent API spec is published.
