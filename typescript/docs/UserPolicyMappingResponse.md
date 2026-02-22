
# UserPolicyMappingResponse

Response for user policy mapping

## Properties

Name | Type
------------ | -------------
`assignedAt` | string
`policyId` | string
`resourceScope` | string
`tenantId` | string
`userId` | string

## Example

```typescript
import type { UserPolicyMappingResponse } from '@tachyon/sdk'

// TODO: Update the object below with actual values
const example = {
  "assignedAt": null,
  "policyId": null,
  "resourceScope": null,
  "tenantId": null,
  "userId": null,
} satisfies UserPolicyMappingResponse

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as UserPolicyMappingResponse
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


