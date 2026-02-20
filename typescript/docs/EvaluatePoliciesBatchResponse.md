
# EvaluatePoliciesBatchResponse

Response for batch policy evaluation

## Properties

Name | Type
------------ | -------------
`results` | [Array&lt;PolicyEvaluationOutcome&gt;](PolicyEvaluationOutcome.md)

## Example

```typescript
import type { EvaluatePoliciesBatchResponse } from '@tachyon/sdk'

// TODO: Update the object below with actual values
const example = {
  "results": null,
} satisfies EvaluatePoliciesBatchResponse

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as EvaluatePoliciesBatchResponse
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


