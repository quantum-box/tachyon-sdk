
# OAuthTokenDetailResponse

Response for OAuth token detail

## Properties

Name | Type
------------ | -------------
`accessToken` | string
`expiresAt` | string
`provider` | string
`providerUserId` | string
`refreshToken` | string

## Example

```typescript
import type { OAuthTokenDetailResponse } from '@tachyon/sdk'

// TODO: Update the object below with actual values
const example = {
  "accessToken": null,
  "expiresAt": null,
  "provider": null,
  "providerUserId": null,
  "refreshToken": null,
} satisfies OAuthTokenDetailResponse

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as OAuthTokenDetailResponse
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


