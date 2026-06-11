
# OAuth2ClientResponse

Response for an OAuth2 client (no secret)

## Properties

Name | Type
------------ | -------------
`allowedScopes` | Array&lt;string&gt;
`authMode` | string
`clientId` | string
`createdAt` | string
`grantTypes` | Array&lt;string&gt;
`id` | string
`name` | string
`redirectUris` | Array&lt;string&gt;
`status` | string
`updatedAt` | string
`useTachyonUserPool` | boolean
`userPoolId` | string

## Example

```typescript
import type { OAuth2ClientResponse } from '@tachyon/sdk'

// TODO: Update the object below with actual values
const example = {
  "allowedScopes": null,
  "authMode": null,
  "clientId": null,
  "createdAt": null,
  "grantTypes": null,
  "id": null,
  "name": null,
  "redirectUris": null,
  "status": null,
  "updatedAt": null,
  "useTachyonUserPool": null,
  "userPoolId": null,
} satisfies OAuth2ClientResponse

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as OAuth2ClientResponse
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


