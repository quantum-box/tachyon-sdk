
# VerifyApiKeyResponse

Response for API key verification

## Properties

Name | Type
------------ | -------------
`name` | string
`serviceAccountId` | string
`tenantId` | string

## Example

```typescript
import type { VerifyApiKeyResponse } from '@tachyon/sdk'

// TODO: Update the object below with actual values
const example = {
  "name": null,
  "serviceAccountId": null,
  "tenantId": null,
} satisfies VerifyApiKeyResponse

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as VerifyApiKeyResponse
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


