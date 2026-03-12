
# UpdateClientRequest

Request to update an OAuth2 client

## Properties

Name | Type
------------ | -------------
`allowedScopes` | Array&lt;string&gt;
`name` | string
`redirectUris` | Array&lt;string&gt;
`status` | string

## Example

```typescript
import type { UpdateClientRequest } from '@tachyon/sdk'

// TODO: Update the object below with actual values
const example = {
  "allowedScopes": null,
  "name": null,
  "redirectUris": null,
  "status": null,
} satisfies UpdateClientRequest

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as UpdateClientRequest
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


