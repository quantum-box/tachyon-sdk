
# AgentExecuteRequest

TODO: add English documentation

## Properties

Name | Type
------------ | -------------
`additionalToolDescription` | string
`agentProtocolId` | string
`agentProtocolMode` | [AgentProtocolMode](AgentProtocolMode.md)
`assistantName` | string
`autoApprove` | boolean
`chatroomNameGeneration` | [ChatroomNameGeneration](ChatroomNameGeneration.md)
`clientTools` | [Array&lt;ClientToolDefinition&gt;](ClientToolDefinition.md)
`maxRequests` | number
`mcpHubConfigJson` | string
`model` | string
`task` | string
`toolAccess` | [AgentToolAccessRequest](AgentToolAccessRequest.md)
`useJsonToolCalls` | boolean
`userCustomInstructions` | string

## Example

```typescript
import type { AgentExecuteRequest } from '@tachyon/sdk'

// TODO: Update the object below with actual values
const example = {
  "additionalToolDescription": null,
  "agentProtocolId": null,
  "agentProtocolMode": null,
  "assistantName": null,
  "autoApprove": null,
  "chatroomNameGeneration": null,
  "clientTools": null,
  "maxRequests": null,
  "mcpHubConfigJson": null,
  "model": null,
  "task": null,
  "toolAccess": null,
  "useJsonToolCalls": null,
  "userCustomInstructions": null,
} satisfies AgentExecuteRequest

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as AgentExecuteRequest
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


