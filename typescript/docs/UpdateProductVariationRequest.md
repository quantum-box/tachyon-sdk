
# UpdateProductVariationRequest

Update variation input

## Properties

Name | Type
------------ | -------------
`code` | string
`currency` | string
`id` | string
`metadata` | any
`name` | string
`publicationDescription` | string
`publicationName` | string
`recurring` | string
`status` | string
`unitAmount` | number

## Example

```typescript
import type { UpdateProductVariationRequest } from '@tachyon/sdk'

// TODO: Update the object below with actual values
const example = {
  "code": null,
  "currency": null,
  "id": null,
  "metadata": null,
  "name": null,
  "publicationDescription": null,
  "publicationName": null,
  "recurring": null,
  "status": null,
  "unitAmount": null,
} satisfies UpdateProductVariationRequest

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as UpdateProductVariationRequest
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


