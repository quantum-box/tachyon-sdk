
# ProductListResponse

Paginated product list response

## Properties

Name | Type
------------ | -------------
`hasNextPage` | boolean
`items` | [Array&lt;ProductResponse&gt;](ProductResponse.md)
`limit` | number
`offset` | number
`totalCount` | number

## Example

```typescript
import type { ProductListResponse } from '@tachyon/sdk'

// TODO: Update the object below with actual values
const example = {
  "hasNextPage": null,
  "items": null,
  "limit": null,
  "offset": null,
  "totalCount": null,
} satisfies ProductListResponse

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as ProductListResponse
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


