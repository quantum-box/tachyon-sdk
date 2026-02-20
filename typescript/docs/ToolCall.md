
# ToolCall

Represents a tool call event

## Properties

Name | Type
------------ | -------------
`toolId` | string
`toolName` | string

## Example

```typescript
import type { ToolCall } from '@tachyon/sdk'

// TODO: Update the object below with actual values
const example = {
  "toolId": null,
  "toolName": null,
} satisfies ToolCall

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as ToolCall
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


