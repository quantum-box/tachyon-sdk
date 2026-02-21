
# ProductVariantResponse

Product variant response

## Properties

Name | Type
------------ | -------------
`code` | string
`createdAt` | Date
`id` | string
`metadata` | any
`name` | string
`productId` | string
`status` | string
`tenantId` | string
`updatedAt` | Date

## Example

```typescript
import type { ProductVariantResponse } from '@tachyon/sdk'

// TODO: Update the object below with actual values
const example = {
  "code": null,
  "createdAt": null,
  "id": null,
  "metadata": null,
  "name": null,
  "productId": null,
  "status": null,
  "tenantId": null,
  "updatedAt": null,
} satisfies ProductVariantResponse

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as ProductVariantResponse
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


