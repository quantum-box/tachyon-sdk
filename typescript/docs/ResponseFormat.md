
# ResponseFormat

Response format type for completion responses

## Properties

Name | Type
------------ | -------------
`type` | string

## Example

```typescript
import type { ResponseFormat } from '@tachyon/sdk'

// TODO: Update the object below with actual values
const example = {
  "type": text,
} satisfies ResponseFormat

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as ResponseFormat
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


