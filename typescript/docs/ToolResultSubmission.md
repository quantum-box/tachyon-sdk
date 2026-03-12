
# ToolResultSubmission

Result submitted by the client for a pending tool call.  After receiving a `tool_call_pending` SSE event, the client executes the tool and submits the result via `POST /v1/llms/chatrooms/{chatroom_id}/agent/tool-result`.  # Example  ```json {   \"tool_id\": \"ct_01JEXAMPLE\",   \"result\": \"{\\\"rows\\\": [{\\\"id\\\": 1, \\\"name\\\": \\\"Alice\\\"}]}\",   \"is_error\": false } ```

## Properties

Name | Type
------------ | -------------
`isError` | boolean
`result` | string
`toolId` | string

## Example

```typescript
import type { ToolResultSubmission } from '@tachyon/sdk'

// TODO: Update the object below with actual values
const example = {
  "isError": null,
  "result": null,
  "toolId": null,
} satisfies ToolResultSubmission

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as ToolResultSubmission
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


