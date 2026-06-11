
# OAuth2CreateClientRequest

Request to create an OAuth2 client

## Properties

Name | Type
------------ | -------------
`allowedScopes` | Array&lt;string&gt;
`authMode` | string
`grantTypes` | Array&lt;string&gt;
`name` | string
`redirectUris` | Array&lt;string&gt;
`useTachyonUserPool` | boolean
`userPoolId` | string

## Example

```typescript
import type { OAuth2CreateClientRequest } from '@tachyon/sdk'

// TODO: Update the object below with actual values
const example = {
  "allowedScopes": null,
  "authMode": null,
  "grantTypes": null,
  "name": null,
  "redirectUris": null,
  "useTachyonUserPool": null,
  "userPoolId": null,
} satisfies OAuth2CreateClientRequest

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as OAuth2CreateClientRequest
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


