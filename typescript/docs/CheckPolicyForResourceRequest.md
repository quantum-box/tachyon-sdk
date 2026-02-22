
# CheckPolicyForResourceRequest

Request to check policy for a specific resource

## Properties

Name | Type
------------ | -------------
`action` | string
`resourceTrn` | string

## Example

```typescript
import type { CheckPolicyForResourceRequest } from '@tachyon/sdk'

// TODO: Update the object below with actual values
const example = {
  "action": null,
  "resourceTrn": null,
} satisfies CheckPolicyForResourceRequest

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as CheckPolicyForResourceRequest
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


