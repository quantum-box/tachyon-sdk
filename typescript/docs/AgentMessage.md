
# AgentMessage

エージェントメッセージレスポンス用のモデル

## Properties

Name | Type
------------ | -------------
`chatroomId` | string
`content` | string
`createdAt` | Date
`id` | string
`messageType` | string
`role` | string
`userId` | string

## Example

```typescript
import type { AgentMessage } from '@tachyon/sdk'

// TODO: Update the object below with actual values
const example = {
  "chatroomId": null,
  "content": null,
  "createdAt": null,
  "id": null,
  "messageType": null,
  "role": null,
  "userId": null,
} satisfies AgentMessage

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as AgentMessage
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


