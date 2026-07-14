# tachyon_sdk.NotificationsApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**send_sms_notification**](NotificationsApi.md#send_sms_notification) | **POST** /v1/notifications/sms | Send an SMS notification


# **send_sms_notification**
> SendSmsNotificationResponse send_sms_notification(x_operator_id, authorization, send_sms_notification_request)

Send an SMS notification

Sends a plain text SMS notification through Tachyon notification providers. Provider-specific services such as AWS SNS are not exposed through the SDK API.

### Example


```python
import tachyon_sdk
from tachyon_sdk.models.send_sms_notification_request import SendSmsNotificationRequest
from tachyon_sdk.models.send_sms_notification_response import SendSmsNotificationResponse
from tachyon_sdk.rest import ApiException
from pprint import pprint

# Defining the host is optional and defaults to http://localhost
# See configuration.py for a list of all supported configuration parameters.
configuration = tachyon_sdk.Configuration(
    host = "http://localhost"
)


# Enter a context with an instance of the API client
with tachyon_sdk.ApiClient(configuration) as api_client:
    # Create an instance of the API class
    api_instance = tachyon_sdk.NotificationsApi(api_client)
    x_operator_id = 'tn_xxxxxx' # str | Operator ID
    authorization = 'Bearer xxxxx' # str | Authorization
    send_sms_notification_request = tachyon_sdk.SendSmsNotificationRequest() # SendSmsNotificationRequest | 

    try:
        # Send an SMS notification
        api_response = api_instance.send_sms_notification(x_operator_id, authorization, send_sms_notification_request)
        print("The response of NotificationsApi->send_sms_notification:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling NotificationsApi->send_sms_notification: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **x_operator_id** | **str**| Operator ID | 
 **authorization** | **str**| Authorization | 
 **send_sms_notification_request** | [**SendSmsNotificationRequest**](SendSmsNotificationRequest.md)|  | 

### Return type

[**SendSmsNotificationResponse**](SendSmsNotificationResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**202** | SMS notification accepted |  -  |
**400** | Bad request |  -  |
**401** | Unauthorized |  -  |
**403** | Forbidden |  -  |
**500** | Internal server error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

