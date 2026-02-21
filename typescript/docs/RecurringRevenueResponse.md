
# RecurringRevenueResponse

Recurring revenue response

## Properties

Name | Type
------------ | -------------
`amount` | number
`changePercentage` | number
`createdAt` | Date
`cycle` | string
`endDate` | string
`id` | string
`startDate` | string
`tenantId` | string

## Example

```typescript
import type { RecurringRevenueResponse } from '@tachyon/sdk'

// TODO: Update the object below with actual values
const example = {
  "amount": null,
  "changePercentage": null,
  "createdAt": null,
  "cycle": null,
  "endDate": null,
  "id": null,
  "startDate": null,
  "tenantId": null,
} satisfies RecurringRevenueResponse

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as RecurringRevenueResponse
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


