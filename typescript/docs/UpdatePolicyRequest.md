
# UpdatePolicyRequest

Request to update a custom policy

## Properties

Name | Type
------------ | -------------
`actionPatternsToAdd` | [Array&lt;PolicyActionPatternRequest&gt;](PolicyActionPatternRequest.md)
`actionPatternsToRemove` | [Array&lt;RemovePolicyActionPatternRequest&gt;](RemovePolicyActionPatternRequest.md)
`actionsToAdd` | [Array&lt;PolicyActionRequest&gt;](PolicyActionRequest.md)
`actionsToRemove` | Array&lt;string&gt;
`description` | string

## Example

```typescript
import type { UpdatePolicyRequest } from '@tachyon/sdk'

// TODO: Update the object below with actual values
const example = {
  "actionPatternsToAdd": null,
  "actionPatternsToRemove": null,
  "actionsToAdd": null,
  "actionsToRemove": null,
  "description": null,
} satisfies UpdatePolicyRequest

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as UpdatePolicyRequest
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


