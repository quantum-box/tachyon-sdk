
# PurchaseOrderResponse

Purchase order response

## Properties

Name | Type
------------ | -------------
`billingInfo` | string
`clientId` | string
`currency` | string
`deliveryDate` | string
`id` | string
`invoiceAddress` | string
`lineItems` | [Array&lt;LineItemResponse&gt;](LineItemResponse.md)
`orderDate` | string
`quotesId` | string
`softwareTenantId` | string
`status` | string
`subtotal` | number
`tax` | number
`tenantId` | string
`total` | number

## Example

```typescript
import type { PurchaseOrderResponse } from '@tachyon/sdk'

// TODO: Update the object below with actual values
const example = {
  "billingInfo": null,
  "clientId": null,
  "currency": null,
  "deliveryDate": null,
  "id": null,
  "invoiceAddress": null,
  "lineItems": null,
  "orderDate": null,
  "quotesId": null,
  "softwareTenantId": null,
  "status": null,
  "subtotal": null,
  "tax": null,
  "tenantId": null,
  "total": null,
} satisfies PurchaseOrderResponse

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as PurchaseOrderResponse
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


