
# CreateChatRoomRequest

Create chatroom request

## Properties

Name | Type
------------ | -------------
`metadata` | [](.md)
`name` | string

## Example

```typescript
import type { CreateChatRoomRequest } from '@tachyon/sdk'

// TODO: Update the object below with actual values
const example = {
  "metadata": null,
  "name": New Room,
} satisfies CreateChatRoomRequest

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as CreateChatRoomRequest
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


