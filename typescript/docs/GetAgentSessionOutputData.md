
# GetAgentSessionOutputData

Output containing the session detail.

## Properties

Name | Type
------------ | -------------
`createdAt` | Date
`id` | string
`metadata` | any
`name` | string
`updatedAt` | Date

## Example

```typescript
import type { GetAgentSessionOutputData } from '@tachyon/sdk'

// TODO: Update the object below with actual values
const example = {
  "createdAt": null,
  "id": null,
  "metadata": null,
  "name": null,
  "updatedAt": null,
} satisfies GetAgentSessionOutputData

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as GetAgentSessionOutputData
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


