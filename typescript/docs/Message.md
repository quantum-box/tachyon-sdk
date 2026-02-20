
# Message

Message type for request/response handling

## Properties

Name | Type
------------ | -------------
`content` | [MessageContent](MessageContent.md)
`role` | string
`toolCalls` | [Array&lt;ToolCallResponse&gt;](ToolCallResponse.md)

## Example

```typescript
import type { Message } from '@tachyon/sdk'

// TODO: Update the object below with actual values
const example = {
  "content": null,
  "role": user,
  "toolCalls": null,
} satisfies Message

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as Message
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


