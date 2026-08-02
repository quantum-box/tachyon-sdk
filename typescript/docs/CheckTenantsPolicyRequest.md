
# CheckTenantsPolicyRequest

Request to evaluate one action across multiple tenant scopes.

## Properties

Name | Type
------------ | -------------
`action` | string
`platformId` | string
`tenantIds` | Array&lt;string&gt;

## Example

```typescript
import type { CheckTenantsPolicyRequest } from '@tachyon/sdk'

// TODO: Update the object below with actual values
const example = {
  "action": null,
  "platformId": null,
  "tenantIds": null,
} satisfies CheckTenantsPolicyRequest

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as CheckTenantsPolicyRequest
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


