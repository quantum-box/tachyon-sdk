
# ChatRoom


## Properties

Name | Type
------------ | -------------
`createdAt` | Date
`id` | string
`metadata` | any
`name` | string
`operatorId` | string
`ownerId` | string
`updatedAt` | Date

## Example

```typescript
import type { ChatRoom } from '@tachyon/sdk'

// TODO: Update the object below with actual values
const example = {
  "createdAt": null,
  "id": null,
  "metadata": null,
  "name": null,
  "operatorId": null,
  "ownerId": null,
  "updatedAt": null,
} satisfies ChatRoom

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as ChatRoom
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


