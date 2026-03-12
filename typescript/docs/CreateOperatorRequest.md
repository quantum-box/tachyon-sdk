
# CreateOperatorRequest

Request body for creating an operator

## Properties

Name | Type
------------ | -------------
`newOperatorOwnerId` | string
`newOperatorOwnerMethod` | string
`newOperatorOwnerPassword` | string
`operatorAlias` | string
`operatorName` | string
`platformId` | string

## Example

```typescript
import type { CreateOperatorRequest } from '@tachyon/sdk'

// TODO: Update the object below with actual values
const example = {
  "newOperatorOwnerId": null,
  "newOperatorOwnerMethod": null,
  "newOperatorOwnerPassword": null,
  "operatorAlias": null,
  "operatorName": null,
  "platformId": null,
} satisfies CreateOperatorRequest

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as CreateOperatorRequest
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


