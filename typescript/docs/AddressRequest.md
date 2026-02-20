
# AddressRequest

Address input

## Properties

Name | Type
------------ | -------------
`address1` | string
`address2` | string
`city` | string
`postalCode` | string
`state` | string

## Example

```typescript
import type { AddressRequest } from '@tachyon/sdk'

// TODO: Update the object below with actual values
const example = {
  "address1": null,
  "address2": null,
  "city": null,
  "postalCode": null,
  "state": null,
} satisfies AddressRequest

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as AddressRequest
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


