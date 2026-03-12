
# CreateAgentSessionOutputData

Output after session creation.

## Properties

Name | Type
------------ | -------------
`createdAt` | Date
`name` | string
`sessionId` | string

## Example

```typescript
import type { CreateAgentSessionOutputData } from '@tachyon/sdk'

// TODO: Update the object below with actual values
const example = {
  "createdAt": null,
  "name": null,
  "sessionId": null,
} satisfies CreateAgentSessionOutputData

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as CreateAgentSessionOutputData
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


