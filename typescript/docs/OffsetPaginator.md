
# OffsetPaginator


## Properties

Name | Type
------------ | -------------
`currentPage` | number
`itemsPerPage` | number
`totalItems` | number
`totalPages` | number

## Example

```typescript
import type { OffsetPaginator } from '@tachyon/sdk'

// TODO: Update the object below with actual values
const example = {
  "currentPage": null,
  "itemsPerPage": null,
  "totalItems": null,
  "totalPages": null,
} satisfies OffsetPaginator

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as OffsetPaginator
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


