
# CreateUserRequest


## Properties

Name | Type
------------ | -------------
`email` | string
`image` | string
`operatorId` | string
`password` | string
`role` | string
`username` | string

## Example

```typescript
import type { CreateUserRequest } from '@tachyon/sdk'

// TODO: Update the object below with actual values
const example = {
  "email": null,
  "image": null,
  "operatorId": null,
  "password": null,
  "role": null,
  "username": null,
} satisfies CreateUserRequest

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as CreateUserRequest
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


