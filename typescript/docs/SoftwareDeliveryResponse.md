
# SoftwareDeliveryResponse

Response for software delivery

## Properties

Name | Type
------------ | -------------
`accessUrl` | string
`createdAt` | string
`deliveredAt` | string
`id` | string
`orderId` | string
`status` | string
`updatedAt` | string

## Example

```typescript
import type { SoftwareDeliveryResponse } from '@tachyon/sdk'

// TODO: Update the object below with actual values
const example = {
  "accessUrl": null,
  "createdAt": null,
  "deliveredAt": null,
  "id": null,
  "orderId": null,
  "status": null,
  "updatedAt": null,
} satisfies SoftwareDeliveryResponse

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as SoftwareDeliveryResponse
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


