
# CreateClientResponse

Response for newly created OAuth2 client (includes plain-text secret)

## Properties

Name | Type
------------ | -------------
`allowedScopes` | Array&lt;string&gt;
`clientId` | string
`clientSecret` | string
`createdAt` | string
`grantTypes` | Array&lt;string&gt;
`id` | string
`name` | string
`redirectUris` | Array&lt;string&gt;
`status` | string
`useTachyonUserPool` | boolean

## Example

```typescript
import type { CreateClientResponse } from '@tachyon/sdk'

// TODO: Update the object below with actual values
const example = {
  "allowedScopes": null,
  "clientId": null,
  "clientSecret": null,
  "createdAt": null,
  "grantTypes": null,
  "id": null,
  "name": null,
  "redirectUris": null,
  "status": null,
  "useTachyonUserPool": null,
} satisfies CreateClientResponse

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as CreateClientResponse
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


