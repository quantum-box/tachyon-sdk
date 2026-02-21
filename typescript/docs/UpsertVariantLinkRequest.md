
# UpsertVariantLinkRequest

Request body for upserting a variant procurement link

## Properties

Name | Type
------------ | -------------
`metadata` | any
`procurementCode` | string
`supplierId` | string
`variantId` | string

## Example

```typescript
import type { UpsertVariantLinkRequest } from '@tachyon/sdk'

// TODO: Update the object below with actual values
const example = {
  "metadata": null,
  "procurementCode": null,
  "supplierId": null,
  "variantId": null,
} satisfies UpsertVariantLinkRequest

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as UpsertVariantLinkRequest
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


