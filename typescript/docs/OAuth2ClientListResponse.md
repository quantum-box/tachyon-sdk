
# OAuth2ClientListResponse

OAuth2 client list response

## Properties

Name | Type
------------ | -------------
`clients` | [Array&lt;OAuth2ClientResponse&gt;](OAuth2ClientResponse.md)

## Example

```typescript
import type { OAuth2ClientListResponse } from '@tachyon/sdk'

// TODO: Update the object below with actual values
const example = {
  "clients": null,
} satisfies OAuth2ClientListResponse

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as OAuth2ClientListResponse
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


