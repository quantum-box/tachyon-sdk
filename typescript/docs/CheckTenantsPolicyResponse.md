
# CheckTenantsPolicyResponse

Response for tenant-bulk policy evaluation.

## Properties

Name | Type
------------ | -------------
`allowedTenants` | [Array&lt;AllowedTenantResponse&gt;](AllowedTenantResponse.md)

## Example

```typescript
import type { CheckTenantsPolicyResponse } from '@tachyon/sdk'

// TODO: Update the object below with actual values
const example = {
  "allowedTenants": null,
} satisfies CheckTenantsPolicyResponse

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as CheckTenantsPolicyResponse
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


