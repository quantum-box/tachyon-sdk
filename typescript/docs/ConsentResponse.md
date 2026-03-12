
# ConsentResponse

Response for a single consent entry.

## Properties

Name | Type
------------ | -------------
`clientId` | string
`clientName` | string
`consentId` | string
`consentedAt` | string
`grantedScopes` | Array&lt;string&gt;
`revokedAt` | string

## Example

```typescript
import type { ConsentResponse } from '@tachyon/sdk'

// TODO: Update the object below with actual values
const example = {
  "clientId": null,
  "clientName": null,
  "consentId": null,
  "consentedAt": null,
  "grantedScopes": null,
  "revokedAt": null,
} satisfies ConsentResponse

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as ConsentResponse
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


