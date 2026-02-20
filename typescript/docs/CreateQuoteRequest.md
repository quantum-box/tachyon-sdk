
# CreateQuoteRequest

Request body for creating a quote

## Properties

Name | Type
------------ | -------------
`clientId` | string
`lineItems` | [Array&lt;LineItemRequest&gt;](LineItemRequest.md)
`title` | string

## Example

```typescript
import type { CreateQuoteRequest } from '@tachyon/sdk'

// TODO: Update the object below with actual values
const example = {
  "clientId": null,
  "lineItems": null,
  "title": null,
} satisfies CreateQuoteRequest

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as CreateQuoteRequest
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


