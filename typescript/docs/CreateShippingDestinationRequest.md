
# CreateShippingDestinationRequest

Request to create a shipping destination

## Properties

Name | Type
------------ | -------------
`address1` | string
`address2` | string
`city` | string
`corporateName` | string
`departmentName` | string
`email` | string
`firstName` | string
`lastName` | string
`phoneNumber` | string
`positionName` | string
`postalCode` | string
`state` | string

## Example

```typescript
import type { CreateShippingDestinationRequest } from '@tachyon/sdk'

// TODO: Update the object below with actual values
const example = {
  "address1": null,
  "address2": null,
  "city": null,
  "corporateName": null,
  "departmentName": null,
  "email": null,
  "firstName": null,
  "lastName": null,
  "phoneNumber": null,
  "positionName": null,
  "postalCode": null,
  "state": null,
} satisfies CreateShippingDestinationRequest

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as CreateShippingDestinationRequest
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


