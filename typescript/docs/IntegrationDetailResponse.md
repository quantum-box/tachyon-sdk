
# IntegrationDetailResponse


## Properties

Name | Type
------------ | -------------
`category` | string
`description` | string
`iconUrl` | string
`id` | string
`isEnabled` | boolean
`isFeatured` | boolean
`name` | string
`provider` | string
`requiresOauth` | boolean
`supportedObjects` | Array&lt;string&gt;
`syncCapability` | string

## Example

```typescript
import type { IntegrationDetailResponse } from '@tachyon/sdk'

// TODO: Update the object below with actual values
const example = {
  "category": null,
  "description": null,
  "iconUrl": null,
  "id": null,
  "isEnabled": null,
  "isFeatured": null,
  "name": null,
  "provider": null,
  "requiresOauth": null,
  "supportedObjects": null,
  "syncCapability": null,
} satisfies IntegrationDetailResponse

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as IntegrationDetailResponse
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


