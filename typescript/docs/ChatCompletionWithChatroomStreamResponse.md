
# ChatCompletionWithChatroomStreamResponse


## Properties

Name | Type
------------ | -------------
`choices` | [Array&lt;ChunkChoice&gt;](ChunkChoice.md)
`created` | number
`id` | string
`model` | string
`object` | string

## Example

```typescript
import type { ChatCompletionWithChatroomStreamResponse } from '@tachyon/sdk'

// TODO: Update the object below with actual values
const example = {
  "choices": null,
  "created": null,
  "id": null,
  "model": null,
  "object": null,
} satisfies ChatCompletionWithChatroomStreamResponse

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as ChatCompletionWithChatroomStreamResponse
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


