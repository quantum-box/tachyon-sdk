
# ToolSchema

Schema type for tool parameters

## Properties

Name | Type
------------ | -------------
`properties` | any
`required` | Array&lt;string&gt;
`type` | string

## Example

```typescript
import type { ToolSchema } from '@tachyon/sdk'

// TODO: Update the object below with actual values
const example = {
  "properties": null,
  "required": null,
  "type": null,
} satisfies ToolSchema

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as ToolSchema
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


