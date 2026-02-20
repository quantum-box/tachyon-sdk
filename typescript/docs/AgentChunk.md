
# AgentChunk

A streaming chunk with optional agent metadata.  When `agent` is `None`, the chunk originates from the main agent. When `Some`, it was relayed from a sub-agent.

## Properties

Name | Type
------------ | -------------
`toolId` | string
`toolName` | string
`type` | string
`args` | any
`isFinished` | boolean
`result` | string
`fireAndForget` | boolean
`index` | number
`text` | string
`createdAt` | Date
`id` | string
`userId` | string
`options` | Array&lt;string&gt;
`command` | string
`cacheCreationInputTokens` | number
`cacheReadInputTokens` | number
`completionTokens` | number
`promptTokens` | number
`totalCost` | number
`totalTokens` | number
`agent` | [AgentSource](AgentSource.md)

## Example

```typescript
import type { AgentChunk } from '@tachyon/sdk'

// TODO: Update the object below with actual values
const example = {
  "toolId": null,
  "toolName": null,
  "type": null,
  "args": null,
  "isFinished": null,
  "result": null,
  "fireAndForget": null,
  "index": null,
  "text": null,
  "createdAt": null,
  "id": null,
  "userId": null,
  "options": null,
  "command": null,
  "cacheCreationInputTokens": null,
  "cacheReadInputTokens": null,
  "completionTokens": null,
  "promptTokens": null,
  "totalCost": null,
  "totalTokens": null,
  "agent": null,
} satisfies AgentChunk

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as AgentChunk
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


