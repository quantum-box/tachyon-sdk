
# ChatCompletionRequest

Request type for chat completion API

## Properties

Name | Type
------------ | -------------
`communicationStyle` | string
`explanationStyle` | string
`frequencyPenalty` | number
`maxCompletionTokens` | number
`memorySettings` | [MemorySettingsRequest](MemorySettingsRequest.md)
`messages` | [Array&lt;Message&gt;](Message.md)
`model` | string
`n` | number
`presencePenalty` | number
`responseFormat` | [ResponseFormat](ResponseFormat.md)
`stream` | boolean
`technicalLevel` | string
`temperature` | number
`toolChoice` | [ToolChoice](ToolChoice.md)
`tools` | [Array&lt;Tool&gt;](Tool.md)
`topP` | number
`user` | string

## Example

```typescript
import type { ChatCompletionRequest } from '@tachyon/sdk'

// TODO: Update the object below with actual values
const example = {
  "communicationStyle": null,
  "explanationStyle": null,
  "frequencyPenalty": null,
  "maxCompletionTokens": null,
  "memorySettings": null,
  "messages": null,
  "model": google_ai:gemini-2.0-flash-exp,
  "n": null,
  "presencePenalty": null,
  "responseFormat": null,
  "stream": null,
  "technicalLevel": null,
  "temperature": null,
  "toolChoice": null,
  "tools": null,
  "topP": null,
  "user": null,
} satisfies ChatCompletionRequest

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as ChatCompletionRequest
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


