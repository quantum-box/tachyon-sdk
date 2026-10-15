
# ContractAuthorityEvidence

Opaque proof metadata returned for a current exact resolution.  The issuer and key identify the server-trusted verification record.  They do not expose private key material, party graphs or the signed token.

## Properties

Name | Type
------------ | -------------
`evidenceDigest` | string
`evidenceId` | string
`evidenceRevision` | string
`issuer` | [AuthorityIssuer](AuthorityIssuer.md)
`revokedAt` | Date
`validFrom` | Date
`validUntil` | Date

## Example

```typescript
import type { ContractAuthorityEvidence } from '@tachyon/sdk'

// TODO: Update the object below with actual values
const example = {
  "evidenceDigest": null,
  "evidenceId": null,
  "evidenceRevision": null,
  "issuer": null,
  "revokedAt": null,
  "validFrom": null,
  "validUntil": null,
} satisfies ContractAuthorityEvidence

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as ContractAuthorityEvidence
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


