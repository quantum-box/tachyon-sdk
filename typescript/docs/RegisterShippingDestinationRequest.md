
# RegisterShippingDestinationRequest

Request body for registering a shipping destination

## Properties

Name | Type
------------ | -------------
`address` | [AddressRequest](AddressRequest.md)
`corporateName` | string
`departmentName` | string
`email` | string
`firstName` | string
`lastName` | string
`phoneNumber` | string
`positionName` | string
`quoteId` | string

## Example

```typescript
import type { RegisterShippingDestinationRequest } from '@tachyon/sdk'

// TODO: Update the object below with actual values
const example = {
  "address": null,
  "corporateName": null,
  "departmentName": null,
  "email": null,
  "firstName": null,
  "lastName": null,
  "phoneNumber": null,
  "positionName": null,
  "quoteId": null,
} satisfies RegisterShippingDestinationRequest

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as RegisterShippingDestinationRequest
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


