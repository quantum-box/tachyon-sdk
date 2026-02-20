
# QuoteResponse

Quote response

## Properties

Name | Type
------------ | -------------
`clientId` | string
`createdAt` | Date
`currency` | string
`id` | string
`lineItems` | [Array&lt;LineItemResponse&gt;](LineItemResponse.md)
`orderDate` | string
`softwareTenantId` | string
`status` | string
`subtotal` | number
`tax` | number
`tenantId` | string
`title` | string
`total` | number
`updatedAt` | Date
`url` | string

## Example

```typescript
import type { QuoteResponse } from '@tachyon/sdk'

// TODO: Update the object below with actual values
const example = {
  "clientId": null,
  "createdAt": null,
  "currency": null,
  "id": null,
  "lineItems": null,
  "orderDate": null,
  "softwareTenantId": null,
  "status": null,
  "subtotal": null,
  "tax": null,
  "tenantId": null,
  "title": null,
  "total": null,
  "updatedAt": null,
  "url": null,
} satisfies QuoteResponse

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as QuoteResponse
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


