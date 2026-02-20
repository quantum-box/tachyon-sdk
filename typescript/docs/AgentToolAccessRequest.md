
# AgentToolAccessRequest


## Properties

Name | Type
------------ | -------------
`agentProtocol` | boolean
`codingAgentJob` | boolean
`command` | boolean
`filesystem` | boolean
`subAgent` | boolean
`urlFetch` | boolean
`webSearch` | boolean

## Example

```typescript
import type { AgentToolAccessRequest } from '@tachyon/sdk'

// TODO: Update the object below with actual values
const example = {
  "agentProtocol": null,
  "codingAgentJob": null,
  "command": null,
  "filesystem": null,
  "subAgent": null,
  "urlFetch": null,
  "webSearch": null,
} satisfies AgentToolAccessRequest

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as AgentToolAccessRequest
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


