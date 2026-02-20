
# UpdateProductRequest

Request body for updating a product

## Properties

Name | Type
------------ | -------------
`billingCycle` | string
`description` | string
`imageFileIds` | Array&lt;string&gt;
`janCode` | string
`kind` | string
`listPrice` | number
`name` | string
`publicationDescription` | string
`publicationName` | string
`publicationStatus` | string
`skuCode` | string
`status` | string
`upcCode` | string
`variations` | [Array&lt;UpdateProductVariationRequest&gt;](UpdateProductVariationRequest.md)

## Example

```typescript
import type { UpdateProductRequest } from '@tachyon/sdk'

// TODO: Update the object below with actual values
const example = {
  "billingCycle": null,
  "description": null,
  "imageFileIds": null,
  "janCode": null,
  "kind": null,
  "listPrice": null,
  "name": null,
  "publicationDescription": null,
  "publicationName": null,
  "publicationStatus": null,
  "skuCode": null,
  "status": null,
  "upcCode": null,
  "variations": null,
} satisfies UpdateProductRequest

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as UpdateProductRequest
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


