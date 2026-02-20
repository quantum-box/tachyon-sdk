
# AgentChunkEventOneOf2

Tool execution result

## Properties

Name | Type
------------ | -------------
`isFinished` | boolean
`result` | string
`toolId` | string
`type` | string

## Example

```typescript
import type { AgentChunkEventOneOf2 } from '@tachyon/sdk'

// TODO: Update the object below with actual values
const example = {
  "isFinished": null,
  "result": null,
  "toolId": null,
  "type": null,
} satisfies AgentChunkEventOneOf2

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as AgentChunkEventOneOf2
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


