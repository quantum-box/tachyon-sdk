
# UpdateQuoteRequest

Request body for updating a quote

## Properties

Name | Type
------------ | -------------
`billingInformationId` | string
`lineItems` | [Array&lt;LineItemRequest&gt;](LineItemRequest.md)
`status` | string
`title` | string

## Example

```typescript
import type { UpdateQuoteRequest } from '@tachyon/sdk'

// TODO: Update the object below with actual values
const example = {
  "billingInformationId": null,
  "lineItems": null,
  "status": null,
  "title": null,
} satisfies UpdateQuoteRequest

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as UpdateQuoteRequest
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


