
# AgentChunkEvent

Agent chunk event types for streaming responses.

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

## Example

```typescript
import type { AgentChunkEvent } from '@tachyon/sdk'

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
} satisfies AgentChunkEvent

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as AgentChunkEvent
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


