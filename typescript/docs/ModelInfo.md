
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
`supportedFeatures` | [Array&lt;SupportedFeature&gt;](SupportedFeature.md)

## Example

```typescript
import type { ModelInfo } from '@tachyon/sdk'

// TODO: Update the object below with actual values
const example = {
  "contextWindow": 200000,
  "description": Claude Sonnet 4 — balanced intelligence and speed,
  "id": anthropic/claude-sonnet-4-20250514,
  "maxOutputTokens": 16384,
  "name": claude-sonnet-4-20250514,
  "provider": anthropic,
  "supportedFeatures": [streaming, function_calling, vision, system_prompt, agent],
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


