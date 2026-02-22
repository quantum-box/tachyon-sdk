
# UserPolicyMappingListResponse

Response for user policy mappings list

## Properties

Name | Type
------------ | -------------
`mappings` | [Array&lt;UserPolicyMappingResponse&gt;](UserPolicyMappingResponse.md)

## Example

```typescript
import type { UserPolicyMappingListResponse } from '@tachyon/sdk'

// TODO: Update the object below with actual values
const example = {
  "mappings": null,
} satisfies UserPolicyMappingListResponse

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as UserPolicyMappingListResponse
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


