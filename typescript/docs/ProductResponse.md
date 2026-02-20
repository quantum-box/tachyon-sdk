
# ProductResponse

Product response

## Properties

Name | Type
------------ | -------------
`billingCycle` | string
`createdAt` | Date
`description` | string
`id` | string
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
`tenantId` | string
`upcCode` | string
`updatedAt` | Date

## Example

```typescript
import type { ProductResponse } from '@tachyon/sdk'

// TODO: Update the object below with actual values
const example = {
  "billingCycle": null,
  "createdAt": null,
  "description": null,
  "id": null,
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
  "tenantId": null,
  "upcCode": null,
  "updatedAt": null,
} satisfies ProductResponse

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as ProductResponse
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


