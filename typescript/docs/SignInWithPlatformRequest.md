
# SignInWithPlatformRequest


## Properties

Name | Type
------------ | -------------
`accessToken` | string
`allowSignUp` | boolean
`email` | string
`name` | string
`platformId` | string

## Example

```typescript
import type { SignInWithPlatformRequest } from '@tachyon/sdk'

// TODO: Update the object below with actual values
const example = {
  "accessToken": null,
  "allowSignUp": null,
  "email": null,
  "name": null,
  "platformId": null,
} satisfies SignInWithPlatformRequest

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as SignInWithPlatformRequest
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


