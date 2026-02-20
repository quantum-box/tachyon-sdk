
# ActionListResponse

Response for action list

## Properties

Name | Type
------------ | -------------
`actions` | [Array&lt;ActionResponse&gt;](ActionResponse.md)
`totalCount` | number

## Example

```typescript
import type { ActionListResponse } from '@tachyon/sdk'

// TODO: Update the object below with actual values
const example = {
  "actions": null,
  "totalCount": null,
} satisfies ActionListResponse

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as ActionListResponse
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


