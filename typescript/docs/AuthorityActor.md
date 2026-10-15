
# AuthorityActor

Actor identity and its source-owned representation.

## Properties

Name | Type
------------ | -------------
`actorId` | string
`representation` | string

## Example

```typescript
import type { AuthorityActor } from '@tachyon/sdk'

// TODO: Update the object below with actual values
const example = {
  "actorId": null,
  "representation": null,
} satisfies AuthorityActor

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as AuthorityActor
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


