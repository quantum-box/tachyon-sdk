
# AgentChunkEventOneOf10

Emitted when a tool job is created, before sync polling begins. Frontend can use the job_id to subscribe to the tool job\'s own SSE stream for real-time progress.

## Properties

Name | Type
------------ | -------------
`jobId` | string
`provider` | string
`toolId` | string
`type` | string

## Example

```typescript
import type { AgentChunkEventOneOf10 } from '@tachyon/sdk'

// TODO: Update the object below with actual values
const example = {
  "jobId": null,
  "provider": null,
  "toolId": null,
  "type": null,
} satisfies AgentChunkEventOneOf10

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as AgentChunkEventOneOf10
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


