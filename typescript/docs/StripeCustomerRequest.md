
# StripeCustomerRequest

Request body for getting or creating a Stripe customer

## Properties

Name | Type
------------ | -------------
`address` | [BillingAddressRequest](BillingAddressRequest.md)
`email` | string
`name` | string
`phone` | string

## Example

```typescript
import type { StripeCustomerRequest } from '@tachyon/sdk'

// TODO: Update the object below with actual values
const example = {
  "address": null,
  "email": null,
  "name": null,
  "phone": null,
} satisfies StripeCustomerRequest

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as StripeCustomerRequest
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


