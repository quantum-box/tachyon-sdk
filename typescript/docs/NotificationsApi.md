# NotificationsApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
|------------- | ------------- | -------------|
| [**sendSmsNotification**](NotificationsApi.md#sendsmsnotificationoperation) | **POST** /v1/notifications/sms | Send an SMS notification |



## sendSmsNotification

> SendSmsNotificationResponse sendSmsNotification(xOperatorId, authorization, sendSmsNotificationRequest)

Send an SMS notification

Sends a plain text SMS notification through Tachyon notification providers. Provider-specific services such as AWS SNS are not exposed through the SDK API.

### Example

```ts
import {
  Configuration,
  NotificationsApi,
} from '@tachyon/sdk';
import type { SendSmsNotificationOperationRequest } from '@tachyon/sdk';

async function example() {
  console.log("🚀 Testing @tachyon/sdk SDK...");
  const api = new NotificationsApi();

  const body = {
    // string | Operator ID
    xOperatorId: tn_xxxxxx,
    // string | Authorization
    authorization: Bearer xxxxx,
    // SendSmsNotificationRequest
    sendSmsNotificationRequest: ...,
  } satisfies SendSmsNotificationOperationRequest;

  try {
    const data = await api.sendSmsNotification(body);
    console.log(data);
  } catch (error) {
    console.error(error);
  }
}

// Run the test
example().catch(console.error);
```

### Parameters


| Name | Type | Description  | Notes |
|------------- | ------------- | ------------- | -------------|
| **xOperatorId** | `string` | Operator ID | [Defaults to `undefined`] |
| **authorization** | `string` | Authorization | [Defaults to `undefined`] |
| **sendSmsNotificationRequest** | [SendSmsNotificationRequest](SendSmsNotificationRequest.md) |  | |

### Return type

[**SendSmsNotificationResponse**](SendSmsNotificationResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: `application/json`
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **202** | SMS notification accepted |  -  |
| **400** | Bad request |  -  |
| **401** | Unauthorized |  -  |
| **403** | Forbidden |  -  |
| **500** | Internal server error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)

