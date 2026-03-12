
# RotateSecretResponse

Response for secret rotation

## Properties

Name | Type
------------ | -------------
`clientId` | string
`newClientSecret` | string

## Example

```typescript
import type { RotateSecretResponse } from '@tachyon/sdk'

// TODO: Update the object below with actual values
const example = {
  "clientId": null,
  "newClientSecret": null,
} satisfies RotateSecretResponse

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as RotateSecretResponse
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


