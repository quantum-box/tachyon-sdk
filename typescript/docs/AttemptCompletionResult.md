
# AttemptCompletionResult

Task completion result

## Properties

Name | Type
------------ | -------------
`command` | string
`isFinished` | boolean
`result` | string

## Example

```typescript
import type { AttemptCompletionResult } from '@tachyon/sdk'

// TODO: Update the object below with actual values
const example = {
  "command": null,
  "isFinished": null,
  "result": null,
} satisfies AttemptCompletionResult

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as AttemptCompletionResult
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


