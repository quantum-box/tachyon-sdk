
# AgentChunkEventOneOf3

Tool call pending client-side execution

## Properties

Name | Type
------------ | -------------
`args` | any
`fireAndForget` | boolean
`toolId` | string
`toolName` | string
`type` | string

## Example

```typescript
import type { AgentChunkEventOneOf3 } from '@tachyon/sdk'

// TODO: Update the object below with actual values
const example = {
  "args": null,
  "fireAndForget": null,
  "toolId": null,
  "toolName": null,
  "type": null,
} satisfies AgentChunkEventOneOf3

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as AgentChunkEventOneOf3
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


