
# UserResponse

Response DTO for a user

## Properties

Name | Type
------------ | -------------
`email` | string
`id` | string
`name` | string
`role` | string
`tenants` | Array&lt;string&gt;

## Example

```typescript
import type { UserResponse } from '@tachyon/sdk'

// TODO: Update the object below with actual values
const example = {
  "email": null,
  "id": null,
  "name": null,
  "role": null,
  "tenants": null,
} satisfies UserResponse

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as UserResponse
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


