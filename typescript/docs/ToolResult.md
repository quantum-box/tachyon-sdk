
# ToolResult

Represents a tool execution result

## Properties

Name | Type
------------ | -------------
`isFinished` | boolean
`result` | string
`toolId` | string

## Example

```typescript
import type { ToolResult } from '@tachyon/sdk'

// TODO: Update the object below with actual values
const example = {
  "isFinished": null,
  "result": null,
  "toolId": null,
} satisfies ToolResult

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as ToolResult
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


