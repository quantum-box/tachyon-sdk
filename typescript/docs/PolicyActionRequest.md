
# PolicyActionRequest

Policy action entry

## Properties

Name | Type
------------ | -------------
`actionFullName` | string
`effect` | string

## Example

```typescript
import type { PolicyActionRequest } from '@tachyon/sdk'

// TODO: Update the object below with actual values
const example = {
  "actionFullName": null,
  "effect": null,
} satisfies PolicyActionRequest

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as PolicyActionRequest
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


