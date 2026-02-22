
# VerifyApiKeyRequest

Request to verify a public API key

## Properties

Name | Type
------------ | -------------
`apiKey` | string
`tenantId` | string

## Example

```typescript
import type { VerifyApiKeyRequest } from '@tachyon/sdk'

// TODO: Update the object below with actual values
const example = {
  "apiKey": null,
  "tenantId": null,
} satisfies VerifyApiKeyRequest

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as VerifyApiKeyRequest
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


