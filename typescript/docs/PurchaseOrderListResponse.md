
# PurchaseOrderListResponse

Purchase order list response

## Properties

Name | Type
------------ | -------------
`items` | [Array&lt;PurchaseOrderResponse&gt;](PurchaseOrderResponse.md)

## Example

```typescript
import type { PurchaseOrderListResponse } from '@tachyon/sdk'

// TODO: Update the object below with actual values
const example = {
  "items": null,
} satisfies PurchaseOrderListResponse

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as PurchaseOrderListResponse
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


