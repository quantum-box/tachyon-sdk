
# ChatCompletionChunkResponse


## Properties

Name | Type
------------ | -------------
`choices` | [Array&lt;ChunkChoice&gt;](ChunkChoice.md)
`created` | number
`id` | string
`model` | string
`object` | string
`systemFingerprint` | string

## Example

```typescript
import type { ChatCompletionChunkResponse } from '@tachyon/sdk'

// TODO: Update the object below with actual values
const example = {
  "choices": null,
  "created": null,
  "id": chatcmpl-123,
  "model": null,
  "object": chat.completion.chunk,
  "systemFingerprint": null,
} satisfies ChatCompletionChunkResponse

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as ChatCompletionChunkResponse
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


