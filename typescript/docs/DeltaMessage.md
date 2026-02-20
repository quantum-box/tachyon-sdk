
# DeltaMessage


## Properties

Name | Type
------------ | -------------
`content` | string
`role` | [ChatRole](ChatRole.md)
`toolCalls` | [Array&lt;ToolCall&gt;](ToolCall.md)

## Example

```typescript
import type { DeltaMessage } from '@tachyon/sdk'

// TODO: Update the object below with actual values
const example = {
  "content": null,
  "role": null,
  "toolCalls": null,
} satisfies DeltaMessage

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as DeltaMessage
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


