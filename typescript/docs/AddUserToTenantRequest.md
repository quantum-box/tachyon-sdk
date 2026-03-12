
# AddUserToTenantRequest

Request DTO for adding a user to a tenant

## Properties

Name | Type
------------ | -------------
`tenantId` | string

## Example

```typescript
import type { AddUserToTenantRequest } from '@tachyon/sdk'

// TODO: Update the object below with actual values
const example = {
  "tenantId": null,
} satisfies AddUserToTenantRequest

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as AddUserToTenantRequest
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


