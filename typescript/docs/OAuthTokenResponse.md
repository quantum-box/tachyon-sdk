
# OAuthTokenResponse

Response for an OAuth token

## Properties

Name | Type
------------ | -------------
`accessToken` | string
`expiresAt` | string
`provider` | string
`providerUserId` | string
`refreshToken` | string
`scope` | string
`tokenType` | string

## Example

```typescript
import type { OAuthTokenResponse } from '@tachyon/sdk'

// TODO: Update the object below with actual values
const example = {
  "accessToken": null,
  "expiresAt": null,
  "provider": null,
  "providerUserId": null,
  "refreshToken": null,
  "scope": null,
  "tokenType": null,
} satisfies OAuthTokenResponse

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as OAuthTokenResponse
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


