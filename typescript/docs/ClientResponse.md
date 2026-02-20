
# ClientResponse

Client response

## Properties

Name | Type
------------ | -------------
`capital` | number
`corporateNumber` | string
`email` | string
`faxNumber` | string
`founded` | string
`headOfficeAddress` | [AddressResponse](AddressResponse.md)
`id` | string
`industry` | string
`listed` | boolean
`name` | string
`phoneNumber` | string
`representative` | string
`tenantId` | string

## Example

```typescript
import type { ClientResponse } from '@tachyon/sdk'

// TODO: Update the object below with actual values
const example = {
  "capital": null,
  "corporateNumber": null,
  "email": null,
  "faxNumber": null,
  "founded": null,
  "headOfficeAddress": null,
  "id": null,
  "industry": null,
  "listed": null,
  "name": null,
  "phoneNumber": null,
  "representative": null,
  "tenantId": null,
} satisfies ClientResponse

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as ClientResponse
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


