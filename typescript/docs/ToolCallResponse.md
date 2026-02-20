
# ToolCallResponse

Tool call response type

## Properties

Name | Type
------------ | -------------
`_function` | [FunctionCallResponse](FunctionCallResponse.md)
`id` | string
`type` | string

## Example

```typescript
import type { ToolCallResponse } from '@tachyon/sdk'

// TODO: Update the object below with actual values
const example = {
  "_function": null,
  "id": null,
  "type": function,
} satisfies ToolCallResponse

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as ToolCallResponse
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


