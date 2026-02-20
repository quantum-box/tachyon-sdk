
# LineItemResponse

Line item response

## Properties

Name | Type
------------ | -------------
`discount` | number
`id` | string
`name` | string
`productId` | string
`quantity` | number
`unitPrice` | number

## Example

```typescript
import type { LineItemResponse } from '@tachyon/sdk'

// TODO: Update the object below with actual values
const example = {
  "discount": null,
  "id": null,
  "name": null,
  "productId": null,
  "quantity": null,
  "unitPrice": null,
} satisfies LineItemResponse

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as LineItemResponse
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


