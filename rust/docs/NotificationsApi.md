# \NotificationsApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**send_sms_notification**](NotificationsApi.md#send_sms_notification) | **POST** /v1/notifications/sms | Send an SMS notification



## send_sms_notification

> models::SendSmsNotificationResponse send_sms_notification(x_operator_id, authorization, send_sms_notification_request)
Send an SMS notification

Sends a plain text SMS notification through Tachyon notification providers. Provider-specific services such as AWS SNS are not exposed through the SDK API.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**x_operator_id** | **String** | Operator ID | [required] |
**authorization** | **String** | Authorization | [required] |
**send_sms_notification_request** | [**SendSmsNotificationRequest**](SendSmsNotificationRequest.md) |  | [required] |

### Return type

[**models::SendSmsNotificationResponse**](SendSmsNotificationResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

