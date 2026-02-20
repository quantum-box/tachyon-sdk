
# ObjectMappingItemResponse

Single item in object mapping list

## Properties

Name | Type
------------ | -------------
`objectMapping` | [ObjectMappingResponse](ObjectMappingResponse.md)
`tenantMapping` | [TenantMappingResponse](TenantMappingResponse.md)

## Example

```typescript
import type { ObjectMappingItemResponse } from '@tachyon/sdk'

// TODO: Update the object below with actual values
const example = {
  "objectMapping": null,
  "tenantMapping": null,
} satisfies ObjectMappingItemResponse

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as ObjectMappingItemResponse
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


