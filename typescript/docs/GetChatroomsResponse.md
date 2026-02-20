
# GetChatroomsResponse


## Properties

Name | Type
------------ | -------------
`chatrooms` | [Array&lt;ChatRoom&gt;](ChatRoom.md)
`paginator` | [OffsetPaginator](OffsetPaginator.md)

## Example

```typescript
import type { GetChatroomsResponse } from '@tachyon/sdk'

// TODO: Update the object below with actual values
const example = {
  "chatrooms": null,
  "paginator": null,
} satisfies GetChatroomsResponse

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as GetChatroomsResponse
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


