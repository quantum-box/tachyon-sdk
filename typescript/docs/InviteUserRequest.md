
# InviteUserRequest

Request DTO for inviting a user

## Properties

Name | Type
------------ | -------------
`inviteeEmail` | string
`inviteeId` | string
`notifyUser` | boolean
`platformId` | string
`tenantId` | string

## Example

```typescript
import type { InviteUserRequest } from '@tachyon/sdk'

// TODO: Update the object below with actual values
const example = {
  "inviteeEmail": null,
  "inviteeId": null,
  "notifyUser": null,
  "platformId": null,
  "tenantId": null,
} satisfies InviteUserRequest

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as InviteUserRequest
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


