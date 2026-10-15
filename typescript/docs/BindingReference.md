
# BindingReference

Exact Field contract identity and binding version/digest.

## Properties

Name | Type
------------ | -------------
`bindingDigest` | string
`bindingRevision` | string
`contractId` | string
`recordOwnerTenantId` | string

## Example

```typescript
import type { BindingReference } from '@tachyon/sdk'

// TODO: Update the object below with actual values
const example = {
  "bindingDigest": null,
  "bindingRevision": null,
  "contractId": null,
  "recordOwnerTenantId": null,
} satisfies BindingReference

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as BindingReference
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


