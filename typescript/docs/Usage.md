
# Usage

Token usage statistics

## Properties

Name | Type
------------ | -------------
`cacheCreationInputTokens` | number
`cacheReadInputTokens` | number
`completionTokens` | number
`promptTokens` | number
`totalCost` | number
`totalTokens` | number

## Example

```typescript
import type { Usage } from '@tachyon/sdk'

// TODO: Update the object below with actual values
const example = {
  "cacheCreationInputTokens": null,
  "cacheReadInputTokens": null,
  "completionTokens": null,
  "promptTokens": null,
  "totalCost": null,
  "totalTokens": null,
} satisfies Usage

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as Usage
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


