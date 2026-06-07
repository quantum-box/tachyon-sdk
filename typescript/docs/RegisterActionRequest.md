
# RegisterActionRequest

Request to register a custom action

## Properties

Name | Type
------------ | -------------
`context` | string
`description` | string
`name` | string
`resourcePattern` | string
`sandboxRestriction` | string

## Example

```typescript
import type { RegisterActionRequest } from '@tachyon/sdk'

// TODO: Update the object below with actual values
const example = {
  "context": null,
  "description": null,
  "name": null,
  "resourcePattern": null,
  "sandboxRestriction": null,
} satisfies RegisterActionRequest

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as RegisterActionRequest
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


