
# ClientListResponse

Client list response

## Properties

Name | Type
------------ | -------------
`items` | [Array&lt;ClientResponse&gt;](ClientResponse.md)

## Example

```typescript
import type { ClientListResponse } from '@tachyon/sdk'

// TODO: Update the object below with actual values
const example = {
  "items": null,
} satisfies ClientListResponse

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as ClientListResponse
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


