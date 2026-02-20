
# AgentChunkEventOneOf

Tool call initiation

## Properties

Name | Type
------------ | -------------
`toolId` | string
`toolName` | string
`type` | string

## Example

```typescript
import type { AgentChunkEventOneOf } from '@tachyon/sdk'

// TODO: Update the object below with actual values
const example = {
  "toolId": null,
  "toolName": null,
  "type": null,
} satisfies AgentChunkEventOneOf

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as AgentChunkEventOneOf
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


