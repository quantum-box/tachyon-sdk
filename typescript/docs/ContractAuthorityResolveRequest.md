
# ContractAuthorityResolveRequest

The current authority request sent by Field.

## Properties

Name | Type
------------ | -------------
`action` | string
`actor` | [AuthorityActor](AuthorityActor.md)
`audience` | string
`binding` | [BindingReference](BindingReference.md)
`environment` | string
`resource` | string
`schemaVersion` | string
`slot` | [BindingSlot](BindingSlot.md)

## Example

```typescript
import type { ContractAuthorityResolveRequest } from '@tachyon/sdk'

// TODO: Update the object below with actual values
const example = {
  "action": null,
  "actor": null,
  "audience": null,
  "binding": null,
  "environment": null,
  "resource": null,
  "schemaVersion": null,
  "slot": null,
} satisfies ContractAuthorityResolveRequest

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as ContractAuthorityResolveRequest
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


