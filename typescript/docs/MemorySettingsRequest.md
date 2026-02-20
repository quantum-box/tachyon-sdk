
# MemorySettingsRequest

Memory settings for context building

## Properties

Name | Type
------------ | -------------
`extractMemory` | boolean
`maxMemories` | number
`minRelevanceScore` | number

## Example

```typescript
import type { MemorySettingsRequest } from '@tachyon/sdk'

// TODO: Update the object below with actual values
const example = {
  "extractMemory": null,
  "maxMemories": null,
  "minRelevanceScore": null,
} satisfies MemorySettingsRequest

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as MemorySettingsRequest
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


