
# ContractAuthorityResolveResponse

Current proof returned for the exact request.  The request is echoed so a consumer can compare every scope field before converting to an internal Field proof.

## Properties

Name | Type
------------ | -------------
`evidence` | [ContractAuthorityEvidence](ContractAuthorityEvidence.md)
`request` | [ContractAuthorityResolveRequest](ContractAuthorityResolveRequest.md)
`schemaVersion` | string

## Example

```typescript
import type { ContractAuthorityResolveResponse } from '@tachyon/sdk'

// TODO: Update the object below with actual values
const example = {
  "evidence": null,
  "request": null,
  "schemaVersion": null,
} satisfies ContractAuthorityResolveResponse

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as ContractAuthorityResolveResponse
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


