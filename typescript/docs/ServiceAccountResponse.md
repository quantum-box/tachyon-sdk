
# ServiceAccountResponse

Response for a service account

## Properties

Name | Type
------------ | -------------
`createdAt` | string
`id` | string
`name` | string
`tenantId` | string

## Example

```typescript
import type { ServiceAccountResponse } from '@tachyon/sdk'

// TODO: Update the object below with actual values
const example = {
  "createdAt": null,
  "id": null,
  "name": null,
  "tenantId": null,
} satisfies ServiceAccountResponse

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as ServiceAccountResponse
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


