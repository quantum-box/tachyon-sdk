
# AttachUserPolicyWithScopeRequest

Request to attach a policy with resource scope

## Properties

Name | Type
------------ | -------------
`policyId` | string
`resourceScope` | string
`tenantId` | string
`userId` | string

## Example

```typescript
import type { AttachUserPolicyWithScopeRequest } from '@tachyon/sdk'

// TODO: Update the object below with actual values
const example = {
  "policyId": null,
  "resourceScope": null,
  "tenantId": null,
  "userId": null,
} satisfies AttachUserPolicyWithScopeRequest

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as AttachUserPolicyWithScopeRequest
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


