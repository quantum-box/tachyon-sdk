
# AllowedTenantResponse

Allowed tenant metadata. Denied tenants and policy internals are omitted.

## Properties

Name | Type
------------ | -------------
`alias` | string
`executionMode` | string
`name` | string
`parentTenantId` | string
`rootTenantId` | string
`tenantId` | string

## Example

```typescript
import type { AllowedTenantResponse } from '@tachyon/sdk'

// TODO: Update the object below with actual values
const example = {
  "alias": null,
  "executionMode": null,
  "name": null,
  "parentTenantId": null,
  "rootTenantId": null,
  "tenantId": null,
} satisfies AllowedTenantResponse

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as AllowedTenantResponse
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)
