
# CreateObjectMappingRequest

Request to create an object mapping

## Properties

Name | Type
------------ | -------------
`entityId` | string
`objectName` | string
`providerName` | string
`providerObjectId` | string

## Example

```typescript
import type { CreateObjectMappingRequest } from '@tachyon/sdk'

// TODO: Update the object below with actual values
const example = {
  "entityId": null,
  "objectName": null,
  "providerName": null,
  "providerObjectId": null,
} satisfies CreateObjectMappingRequest

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as CreateObjectMappingRequest
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


