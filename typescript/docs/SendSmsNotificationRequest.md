
# SendSmsNotificationRequest

Request to send a text SMS notification through Tachyon notification providers.

## Properties

Name | Type
------------ | -------------
`phoneNumber` | string
`message` | string

## Example

```typescript
import type { SendSmsNotificationRequest } from '@tachyon/sdk'

// TODO: Update the object below with actual values
const example = {
  "phoneNumber": +15551234567,
  "message": Your verification code is 123456.,
} satisfies SendSmsNotificationRequest

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as SendSmsNotificationRequest
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


