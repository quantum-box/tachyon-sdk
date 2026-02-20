
# ModelInfo


## Properties

Name | Type
------------ | -------------
`contextWindow` | number
`description` | string
`id` | string
`maxOutputTokens` | number
`name` | string
`provider` | string
`supportedFeatures` | Array&lt;string&gt;

## Example

```typescript
import type { ModelInfo } from '@tachyon/sdk'

// TODO: Update the object below with actual values
const example = {
  "contextWindow": 200000,
  "description": Claude 3 Sonnet model optimized for general-purpose tasks,
  "id": anthropic/claude-3-sonnet-20241022,
  "maxOutputTokens": 4096,
  "name": claude-3-sonnet-20241022,
  "provider": anthropic,
  "supportedFeatures": [chat, streaming, function_calling],
} satisfies ModelInfo

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as ModelInfo
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


