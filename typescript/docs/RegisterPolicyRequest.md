
# RegisterPolicyRequest

Request to register a custom policy

## Properties

Name | Type
------------ | -------------
`actionPatterns` | [Array&lt;PolicyActionPatternRequest&gt;](PolicyActionPatternRequest.md)
`actions` | [Array&lt;PolicyActionRequest&gt;](PolicyActionRequest.md)
`description` | string
`global` | boolean
`isSystem` | boolean
`name` | string
`tenantId` | string

## Example

```typescript
import type { RegisterPolicyRequest } from '@tachyon/sdk'

// TODO: Update the object below with actual values
const example = {
  "actionPatterns": null,
  "actions": null,
  "description": null,
  "global": null,
  "isSystem": null,
  "name": null,
  "tenantId": null,
} satisfies RegisterPolicyRequest

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as RegisterPolicyRequest
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


