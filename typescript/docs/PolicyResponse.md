
# PolicyResponse

Response for a policy

## Properties

Name | Type
------------ | -------------
`createdAt` | string
`description` | string
`id` | string
`isSystem` | boolean
`name` | string
`tenantId` | string
`updatedAt` | string

## Example

```typescript
import type { PolicyResponse } from '@tachyon/sdk'

// TODO: Update the object below with actual values
const example = {
  "createdAt": null,
  "description": null,
  "id": null,
  "isSystem": null,
  "name": null,
  "tenantId": null,
  "updatedAt": null,
} satisfies PolicyResponse

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as PolicyResponse
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


