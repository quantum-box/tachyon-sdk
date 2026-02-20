
# ToolCallPending

SSE event indicating a client-side tool call is pending.  Emitted when the LLM invokes a user-defined client tool. The client should execute the tool using the provided `args` and submit the result via the tool-result endpoint, unless `fire_and_forget` is `true`.  SSE event name: `tool_call_pending`

## Properties

Name | Type
------------ | -------------
`args` | any
`fireAndForget` | boolean
`toolId` | string
`toolName` | string

## Example

```typescript
import type { ToolCallPending } from '@tachyon/sdk'

// TODO: Update the object below with actual values
const example = {
  "args": null,
  "fireAndForget": null,
  "toolId": null,
  "toolName": null,
} satisfies ToolCallPending

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as ToolCallPending
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


