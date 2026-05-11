
# AgentBuiltinToolRequest


## Properties

Name | Type
------------ | -------------
`name` | string
`type` | [AgentBuiltinToolType](AgentBuiltinToolType.md)

## Example

```typescript
import type { AgentBuiltinToolRequest } from '@tachyon/sdk'

// TODO: Update the object below with actual values
const example = {
  "name": null,
  "type": null,
} satisfies AgentBuiltinToolRequest

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as AgentBuiltinToolRequest
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


