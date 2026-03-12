
# ConnectionResponse


## Properties

Name | Type
------------ | -------------
`connectedAt` | string
`errorMessage` | string
`externalAccountId` | string
`externalAccountName` | string
`id` | string
`integrationId` | string
`lastSyncedAt` | string
`metadata` | { [key: string]: any; }
`provider` | string
`status` | string

## Example

```typescript
import type { ConnectionResponse } from '@tachyon/sdk'

// TODO: Update the object below with actual values
const example = {
  "connectedAt": null,
  "errorMessage": null,
  "externalAccountId": null,
  "externalAccountName": null,
  "id": null,
  "integrationId": null,
  "lastSyncedAt": null,
  "metadata": null,
  "provider": null,
  "status": null,
} satisfies ConnectionResponse

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as ConnectionResponse
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


