# SendSmsNotificationResponse

Response returned after an SMS notification send request is accepted.

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**accepted** | **bool** | Whether Tachyon accepted the SMS notification for delivery. | 

## Example

```python
from tachyon_sdk.models.send_sms_notification_response import SendSmsNotificationResponse

# TODO update the JSON string below
json = "{}"
# create an instance of SendSmsNotificationResponse from a JSON string
send_sms_notification_response_instance = SendSmsNotificationResponse.from_json(json)
# print the JSON string representation of the object
print(SendSmsNotificationResponse.to_json())

# convert the object into a dict
send_sms_notification_response_dict = send_sms_notification_response_instance.to_dict()
# create an instance of SendSmsNotificationResponse from a dict
send_sms_notification_response_from_dict = SendSmsNotificationResponse.from_dict(send_sms_notification_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


