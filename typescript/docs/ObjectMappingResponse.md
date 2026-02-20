
# ObjectMappingResponse

Response for an object mapping

## Properties

Name | Type
------------ | -------------
`createdAt` | string
`entityId` | string
`objectName` | string
`providerName` | string
`providerPrimaryId` | string
`tenantId` | string
`updatedAt` | string

## Example

```typescript
import type { ObjectMappingResponse } from '@tachyon/sdk'

// TODO: Update the object below with actual values
const example = {
  "createdAt": null,
  "entityId": null,
  "objectName": null,
  "providerName": null,
  "providerPrimaryId": null,
  "tenantId": null,
  "updatedAt": null,
} satisfies ObjectMappingResponse

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as ObjectMappingResponse
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


