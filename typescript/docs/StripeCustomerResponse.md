
# StripeCustomerResponse

Stripe customer response

## Properties

Name | Type
------------ | -------------
`defaultPaymentMethodId` | string
`stripeCustomerId` | string
`stripeSubscriptionId` | string
`stripeSubscriptionItemId` | string

## Example

```typescript
import type { StripeCustomerResponse } from '@tachyon/sdk'

// TODO: Update the object below with actual values
const example = {
  "defaultPaymentMethodId": null,
  "stripeCustomerId": null,
  "stripeSubscriptionId": null,
  "stripeSubscriptionItemId": null,
} satisfies StripeCustomerResponse

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as StripeCustomerResponse
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


