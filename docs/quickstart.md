# Tachyon Agent SDK — Quickstart

## Installation

```bash
npm install @tachyon-sdk/agent
```

## Requirements

- Node.js 18+
- A Tachyon API key (`TACHYON_API_KEY`)
- The Tachyon API base URL (e.g. `https://api.n1.tachy.one`)

---

## `streamAgent` — Convenience Function

The quickest way to stream agent responses. Creates a chat room internally and streams events.

```js
import { streamAgent } from '@tachyon-sdk/agent'

const apiKey = process.env.TACHYON_API_KEY
const baseUrl = process.env.TACHYON_API_URL ?? 'https://api.n1.tachy.one'

for await (const event of streamAgent('Slackに通知して', { apiKey, baseUrl })) {
  switch (event.type) {
    case 'text':
      process.stdout.write(event.content)
      break
    case 'tool_call':
      console.error(`[tool] ${event.content}`)
      break
    case 'done':
      console.log('\nDone.')
      break
  }
}
```

### `StreamAgentOptions`

| Option        | Type     | Description                                      |
|---------------|----------|--------------------------------------------------|
| `apiKey`      | `string` | API access token (alias for `accessToken`)       |
| `accessToken` | `string` | API access token                                 |
| `baseUrl`     | `string` | API base URL (alias for `apiBaseUrl`)            |
| `apiBaseUrl`  | `string` | API base URL                                     |
| `tenantId`    | `string` | Optional operator tenant ID (`x-operator-id`)   |
| `userId`      | `string` | Optional user ID (`x-user-id`)                  |

### `StreamEvent` types

| `type`      | `content`                        |
|-------------|----------------------------------|
| `text`      | Text output from the agent       |
| `tool_call` | Name of the tool being called    |
| `done`      | Stream completed (no `content`)  |

---

## `TachyonAgentClient` — Class API

For full control, use the client class directly.

```typescript
import { TachyonAgentClient } from '@tachyon-sdk/agent'

const client = new TachyonAgentClient({
  apiBaseUrl: 'https://api.n1.tachy.one',
  accessToken: process.env.TACHYON_API_KEY!,
  tenantId: 'tn_xxx',   // optional
  userId: 'user_xxx',   // optional
})

// Create a chat room
const chatroom = await client.createChatRoom('My session')

// Stream agent execution
for await (const chunk of client.stream(chatroom.id, { task: 'Hello!' })) {
  if (chunk.type === 'say') {
    process.stdout.write(chunk.text ?? '')
  }
}

// Non-streaming execution (collects all chunks)
const chunks = await client.execute(chatroom.id, { task: 'Hello!' })

// Retrieve stored messages
const messages = await client.getMessages(chatroom.id)
```

### `TachyonAgentClientConfig`

| Field         | Type     | Required | Description                        |
|---------------|----------|----------|------------------------------------|
| `apiBaseUrl`  | `string` | Yes      | API base URL                       |
| `accessToken` | `string` | Yes      | Bearer token for authentication    |
| `tenantId`    | `string` | No       | Operator tenant ID (`x-operator-id`) |
| `userId`      | `string` | No       | User ID header (`x-user-id`)       |

### Methods

| Method                                          | Returns                    | Description                              |
|-------------------------------------------------|----------------------------|------------------------------------------|
| `createChatRoom(title?)`                        | `Promise<ChatRoom>`        | Create a new chat room                   |
| `stream(chatroomId, request)`                   | `AsyncGenerator<AgentChunk>` | Stream agent execution via SSE         |
| `execute(chatroomId, request)`                  | `Promise<AgentChunk[]>`    | Non-streaming execution (all chunks)     |
| `getMessages(chatroomId)`                       | `Promise<AgentChunk[]>`    | Retrieve stored messages for a chat room |

---

## TypeScript Types

```typescript
import type {
  AgentChunk,
  AgentExecuteRequest,
  ChatRoom,
  StreamEvent,
  StreamAgentOptions,
  TachyonAgentClientConfig,
} from '@tachyon-sdk/agent'
```

### `AgentExecuteRequest`

| Field                      | Type      | Description                        |
|----------------------------|-----------|------------------------------------|
| `task`                     | `string`  | The prompt/task for the agent      |
| `model`                    | `string`  | Optional model override            |
| `auto_approve`             | `boolean` | Auto-approve tool calls            |
| `max_requests`             | `number`  | Max number of LLM requests         |
| `tool_access`              | `object`  | Tool access permissions            |
| `agent_protocol_id`        | `string`  | Agent protocol ID                  |
| `user_custom_instructions` | `string`  | Custom instructions for the agent  |

### Error Handling

```typescript
import { TachyonAgentError } from '@tachyon-sdk/agent'

try {
  const chatroom = await client.createChatRoom()
} catch (err) {
  if (err instanceof TachyonAgentError) {
    console.error(`Status: ${err.status}, Body: ${err.body}`)
  }
}
```
