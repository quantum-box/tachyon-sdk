
# BillingAddressRequest

Billing address input

## Properties

Name | Type
------------ | -------------
`city` | string
`country` | string
`line1` | string
`line2` | string
`postalCode` | string
`state` | string

## Example

```typescript
import type { BillingAddressRequest } from '@tachyon/sdk'

// TODO: Update the object below with actual values
const example = {
  "city": null,
  "country": null,
  "line1": null,
  "line2": null,
  "postalCode": null,
  "state": null,
} satisfies BillingAddressRequest

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as BillingAddressRequest
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


