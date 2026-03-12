
# AgentSessionEntry

Single session entry in the list response.

## Properties

Name | Type
------------ | -------------
`createdAt` | Date
`id` | string
`name` | string
`updatedAt` | Date

## Example

```typescript
import type { AgentSessionEntry } from '@tachyon/sdk'

// TODO: Update the object below with actual values
const example = {
  "createdAt": null,
  "id": null,
  "name": null,
  "updatedAt": null,
} satisfies AgentSessionEntry

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as AgentSessionEntry
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


