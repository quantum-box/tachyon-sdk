# @tachyon-sdk/agent-chat

React component library and API client for Tachyon agent chat.

## Installation

```bash
npm install @tachyon-sdk/agent-chat
```

## Quick Start — Next.js App Router

### 1. Wrap your layout with `AgentChatProvider`

```tsx
// app/layout.tsx
import { AgentChatProvider } from '@tachyon-sdk/agent-chat'

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en">
      <body>
        <AgentChatProvider
          apiBaseUrl={process.env.NEXT_PUBLIC_TACHYON_API_URL!}
          accessToken={process.env.NEXT_PUBLIC_TACHYON_ACCESS_TOKEN!}
          tenantId={process.env.NEXT_PUBLIC_TACHYON_TENANT_ID!}
          defaultModel="anthropic/claude-sonnet-4.5"
        >
          {children}
        </AgentChatProvider>
      </body>
    </html>
  )
}
```

### 2. Drop in the `AgentChat` component

```tsx
// app/page.tsx
'use client'

import { AgentChat } from '@tachyon-sdk/agent-chat'

export default function Page() {
  // Floating chat button (default)
  return <AgentChat mode="floating" />
}
```

For an inline panel:

```tsx
<AgentChat mode="inline" className="h-[600px]" />
```

#### `AgentChat` props

| Prop          | Type                      | Default      | Description                    |
| ------------- | ------------------------- | ------------ | ------------------------------ |
| `mode`        | `'floating' \| 'inline'`  | `'floating'` | Rendering mode                 |
| `className`   | `string`                  | —            | Additional CSS classes         |
| `panelWidth`  | `number`                  | —            | Panel width in px (floating)   |
| `panelHeight` | `number`                  | —            | Panel height in px (floating)  |

---

## `AgentChatProvider` config

`AgentChatProvider` accepts all `AgentChatConfig` props directly (no nested `config` object).

```ts
type AgentChatConfig = {
  // Required — connection
  apiBaseUrl: string       // e.g. "https://api.example.com"
  accessToken: string      // Bearer token
  tenantId: string         // Operator tenant ID (tn_xxx)
  userId?: string          // Optional per-user header

  // Optional — behaviour
  defaultModel?: string    // Default: "anthropic/claude-sonnet-4.5"
  context?: {
    page: string
    hint?: string
    data?: Record<string, unknown>
  }
  toolAccess?: ToolAccess
  onError?: (error: Error) => void
  onMessage?: (message: AgentChunk) => void
}
```

---

## `useAgentChat` hook

Build a fully custom UI with the `useAgentChat` hook. Must be used inside `AgentChatProvider`.

```tsx
'use client'

import { useAgentChat } from '@tachyon-sdk/agent-chat'

export function MyChatUI() {
  const {
    // Message state
    chunks,
    isLoading,
    error,

    // Input
    input,
    setInput,
    handleSubmit,

    // Session management
    sessionId,
    newChat,
    selectSession,

    // Chat room CRUD
    chatRooms,
    fetchChatRooms,
    createChatRoom,
    deleteChatRoom,

    // Model / settings
    selectedModel,
    setSelectedModel,
    maxRequests,
    setMaxRequests,
  } = useAgentChat()

  return (
    <div>
      <ul>
        {chunks.map(chunk => (
          <li key={chunk.id}>
            <strong>{chunk.type}</strong>: {chunk.text ?? chunk.content}
          </li>
        ))}
      </ul>

      {error && <p style={{ color: 'red' }}>{error.message}</p>}

      <form onSubmit={handleSubmit}>
        <input
          value={input}
          onChange={e => setInput(e.target.value)}
          placeholder="Ask something…"
          disabled={isLoading}
        />
        <button type="submit" disabled={isLoading || !input.trim()}>
          {isLoading ? 'Running…' : 'Send'}
        </button>
      </form>

      <button onClick={newChat}>New chat</button>
    </div>
  )
}
```

### Return value

| Field               | Type                                        | Description                                        |
| ------------------- | ------------------------------------------- | -------------------------------------------------- |
| `sessionId`         | `string \| null`                            | Current chatroom ID                                |
| `setSessionId`      | `(id: string \| null) => void`              | Set session directly                               |
| `newChat`           | `() => void`                               | Reset to a blank session                           |
| `selectSession`     | `(id: string) => Promise<void>`            | Switch to an existing chatroom                     |
| `chunks`            | `AgentChunk[]`                              | All streamed/loaded message chunks                 |
| `setChunks`         | `(chunks: AgentChunk[]) => void`            | Override chunks (advanced)                         |
| `isLoading`         | `boolean`                                   | Stream in progress                                 |
| `error`             | `Error \| null`                             | Last error                                         |
| `input`             | `string`                                    | Controlled input value                             |
| `setInput`          | `(v: string) => void`                       | Update input                                       |
| `handleSubmit`      | `(e: React.FormEvent) => Promise<void>`    | Submit handler (clears input, calls sendUserMessage) |
| `sendUserMessage`   | `(message: string) => Promise<boolean>`    | Send a message programmatically                    |
| `startTask`         | `(task: string, title?: string) => Promise<void>` | Execute agent task directly                 |
| `resetChat`         | `() => void`                               | Abort stream and clear all state                   |
| `refetchMessages`   | `() => Promise<void>`                      | Reload messages from server                        |
| `selectedModel`     | `string \| null`                            | Currently selected model ID                        |
| `setSelectedModel`  | `(model: string \| null) => void`           | Change model                                       |
| `maxRequests`       | `number`                                    | Max agent loop iterations (default 10)             |
| `setMaxRequests`    | `(n: number) => void`                       | Change max requests                                |
| `toolAccess`        | `ToolAccess`                                | Tool permission flags                              |
| `setToolAccess`     | `(t: ToolAccess) => void`                   | Update tool permissions                            |
| `chatRooms`         | `ChatRoom[]`                                | List of chatrooms                                  |
| `chatRoomLoading`   | `boolean`                                   | Chatroom list loading state                        |
| `chatRoomError`     | `Error \| null`                             | Chatroom list error                                |
| `fetchChatRooms`    | `() => Promise<void>`                      | Reload chatroom list                               |
| `createChatRoom`    | `(title?: string) => Promise<void>`        | Create a new chatroom                              |
| `updateChatRoom`    | `(id: string, payload: { name?: string }) => Promise<void>` | Rename a chatroom          |
| `deleteChatRoom`    | `(id: string) => Promise<void>`            | Delete a chatroom                                  |

---

## Authentication

`AgentChatProvider` requires an `accessToken` and `tenantId`. The following patterns cover the most common authentication setups.

### Pattern 1 — Tachyon OAuth (`tachyon login`)

Use the token issued by `tachyon login`. In a Next.js app the token is typically stored server-side and injected at request time (see Pattern 3).

```tsx
// app/layout.tsx
import { AgentChatProvider } from '@tachyon-sdk/agent-chat'

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en">
      <body>
        <AgentChatProvider
          apiBaseUrl={process.env.NEXT_PUBLIC_TACHYON_API_URL!}
          accessToken={process.env.NEXT_PUBLIC_TACHYON_ACCESS_TOKEN!} // OAuth token
          tenantId={process.env.NEXT_PUBLIC_TACHYON_TENANT_ID!}
        >
          {children}
        </AgentChatProvider>
      </body>
    </html>
  )
}
```

`NEXT_PUBLIC_TACHYON_ACCESS_TOKEN` should be set to the token obtained via `tachyon login` (stored in `~/.tachyon/credentials` or equivalent).

### Pattern 2 — API Key authentication (`TACHYON_API_KEY`)

Use a long-lived API key issued from the Tachyon operator console. Pass it directly as `accessToken`.

```tsx
// app/layout.tsx
import { AgentChatProvider } from '@tachyon-sdk/agent-chat'

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en">
      <body>
        <AgentChatProvider
          apiBaseUrl={process.env.NEXT_PUBLIC_TACHYON_API_URL!}
          accessToken={process.env.NEXT_PUBLIC_TACHYON_API_KEY!} // API key as bearer token
          tenantId={process.env.NEXT_PUBLIC_TACHYON_TENANT_ID!}
        >
          {children}
        </AgentChatProvider>
      </body>
    </html>
  )
}
```

> **Security note:** `NEXT_PUBLIC_*` variables are embedded in the client bundle. For production use, prefer Pattern 3 below to keep secrets server-side.

### Pattern 3 — Next.js Server Component → Client Component (recommended)

Fetch the token on the server (where secrets stay private) and pass it as a prop to a Client Component wrapper.

```tsx
// app/providers.tsx  — Client Component wrapper
'use client'

import { AgentChatProvider } from '@tachyon-sdk/agent-chat'

type Props = {
  accessToken: string
  tenantId: string
  children: React.ReactNode
}

export function Providers({ accessToken, tenantId, children }: Props) {
  return (
    <AgentChatProvider
      apiBaseUrl={process.env.NEXT_PUBLIC_TACHYON_API_URL!}
      accessToken={accessToken}
      tenantId={tenantId}
    >
      {children}
    </AgentChatProvider>
  )
}
```

```tsx
// app/layout.tsx  — Server Component
import { Providers } from './providers'

async function getAccessToken(): Promise<string> {
  // e.g. read from a session cookie, KV store, or call your auth endpoint
  const res = await fetch('https://your-auth-endpoint/token', {
    headers: { Cookie: /* forward request cookies */ '' },
    cache: 'no-store',
  })
  const data = await res.json()
  return data.access_token
}

export default async function RootLayout({ children }: { children: React.ReactNode }) {
  const accessToken = await getAccessToken()

  return (
    <html lang="en">
      <body>
        <Providers
          accessToken={accessToken}
          tenantId={process.env.TACHYON_TENANT_ID!} // server-side env, not NEXT_PUBLIC_
        >
          {children}
        </Providers>
      </body>
    </html>
  )
}
```

### Accessing config in child components

Use `useAgentChatContext` to read the current client configuration from any component inside `AgentChatProvider`:

```tsx
'use client'

import { useAgentChatContext } from '@tachyon-sdk/agent-chat'

export function DebugPanel() {
  const { client, config } = useAgentChatContext()

  return <p>Connected to {config.apiBaseUrl} as tenant {config.tenantId}</p>
}
```

---

## `AgentChatClient` — direct API access

Use `AgentChatClient` outside of React, in server actions, or when you need low-level control.

```ts
import { AgentChatClient } from '@tachyon-sdk/agent-chat'

const client = new AgentChatClient({
  apiBaseUrl: 'https://api.example.com',
  accessToken: 'your-token',
  tenantId: 'tn_xxxxx',
})

// Create a chatroom and stream a task
const room = await client.createChatRoom('My session')

for await (const chunk of client.streamAgent(room.chatroom.id, {
  task: 'Summarise the latest quarterly report',
  model: 'anthropic/claude-sonnet-4.5',
  max_requests: 5,
  tool_access: { web_search: true },
})) {
  if (chunk.type === 'say' || chunk.type === 'attempt_completion') {
    process.stdout.write(chunk.text ?? '')
  }
}
```

### `AgentChatClientConfig`

| Field         | Type     | Required | Description                          |
| ------------- | -------- | -------- | ------------------------------------ |
| `apiBaseUrl`  | `string` | ✓        | Base URL of the Tachyon API server   |
| `accessToken` | `string` | ✓        | Bearer token for authentication      |
| `tenantId`    | `string` | ✓        | Operator tenant ID (`tn_xxx`)        |
| `userId`      | `string` | —        | Optional per-request user ID header  |

### Key methods

| Method                                         | Description                                      |
| ---------------------------------------------- | ------------------------------------------------ |
| `createChatRoom(title?)`                       | Create a new chatroom                            |
| `getChatrooms()`                               | List all chatrooms                               |
| `deleteChatroom(chatRoomId)`                   | Delete a chatroom                                |
| `streamAgent(chatRoomId, args)`                | Execute agent and yield `AgentChunk` via SSE     |
| `executeAgent(chatRoomId, args)`               | Raw SSE `Response` (for custom streaming)        |
| `getMessages(chatRoomId)`                      | Fetch persisted message history                  |
| `getStatus(chatRoomId)`                        | Poll agent execution status                      |
| `getModels(options?)`                          | List available LLM models                        |
| `submitToolResult(chatRoomId, payload)`        | Submit a human-in-the-loop tool result           |
| `getMemories(status?)`                         | List saved memories                              |
| `createMemory(payload)`                        | Save a new memory                                |
| `getAgentProtocols(options?)`                  | List agent protocols                             |
| `createAgentProtocol(payload)`                 | Create an agent protocol                         |
| `getInsights()`                                | Get user insights                                |
| `createToolJob(payload)`                       | Create a coding-agent tool job                   |
| `streamToolJob(jobId)`                         | Stream tool job output                           |
| `searchTools(payload)`                         | Search available MCP tools                       |

---

## `AgentChunk` type

All streamed messages are `AgentChunk` objects:

```ts
type AgentChunk = {
  type:
    | 'user' | 'ask' | 'thinking'
    | 'tool_call' | 'tool_call_args' | 'tool_result' | 'tool_call_pending'
    | 'say' | 'attempt_completion' | 'assistant' | 'completion'
    | 'usage' | 'tool_job_started'
  id: string
  created_at: string        // ISO 8601
  text?: string
  thinking?: string
  tool_name?: string
  tool_arguments?: string
  tool_result?: string
  tool_id?: string
  // usage chunks
  prompt_tokens?: number
  completion_tokens?: number
  total_tokens?: number
  total_cost?: number
  // tool_job_started chunks
  job_id?: string
  provider?: string
}
```

---

## `ToolAccess` flags

```ts
type ToolAccess = {
  filesystem?: boolean       // Read/write local files
  command?: boolean          // Execute shell commands
  coding_agent_job?: boolean // Spawn coding agent jobs (default: true)
  agent_protocol?: boolean   // Use agent protocols
  web_search?: boolean       // Web search
  url_fetch?: boolean        // Fetch URLs
  sub_agent?: boolean        // Spawn sub-agents
}
```

---

## Available exports

### Components
`AgentChat` · `FloatingChatPanel` · `ChatPanel` · `MessageList` · `MessageBubble` · `ChatInput` · `ModelSelector` · `ThinkingIndicator` · `ToolCallDisplay` · `ToolResultDisplay` · `UsageSummary`

### Hooks
`useAgentChat` · `useAgentStream` · `useChatRoom` · `useSavedMemory` · `useAgentProtocols` · `useUserInsights` · `useToolJobs` · `usePersisted`

### Client & Provider
`AgentChatClient` · `AgentChatProvider` · `useAgentChatContext`

### API factories (advanced)
`createChatroomApi` · `createAgentApi` · `createMemoryApi` · `createProtocolApi` · `createInsightApi` · `createToolJobApi` · `createToolSearchApi`
