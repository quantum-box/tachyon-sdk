# AgentToolAccessRequest

Builtin tools enabled for agent execution. Omitted tools are disabled.

## Type

```typescript
Array<{
  type: 'builtin'
  name: string
}>
```

## Example

```typescript
import type { AgentToolAccessRequest } from '@tachyon/sdk'

const example = [
  { type: 'builtin', name: 'filesystem' },
  { type: 'builtin', name: 'web_search' },
  { type: 'builtin', name: 'url_fetch' },
] satisfies AgentToolAccessRequest

console.log(example)

const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

const exampleParsed = JSON.parse(exampleJSON) as AgentToolAccessRequest
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)
