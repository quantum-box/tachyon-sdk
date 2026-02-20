
# CreateClientRequest

Request body for creating a client

## Properties

Name | Type
------------ | -------------
`address` | [AddressRequest](AddressRequest.md)
`capital` | number
`corporateNumber` | string
`createCrm` | boolean
`email` | string
`faxNumber` | string
`founded` | string
`id` | string
`industry` | string
`listed` | boolean
`name` | string
`phoneNumber` | string
`representative` | string

## Example

```typescript
import type { CreateClientRequest } from '@tachyon/sdk'

// TODO: Update the object below with actual values
const example = {
  "address": null,
  "capital": null,
  "corporateNumber": null,
  "createCrm": null,
  "email": null,
  "faxNumber": null,
  "founded": null,
  "id": null,
  "industry": null,
  "listed": null,
  "name": null,
  "phoneNumber": null,
  "representative": null,
} satisfies CreateClientRequest

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as CreateClientRequest
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


