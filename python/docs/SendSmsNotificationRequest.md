# SendSmsNotificationRequest

Request to send a text SMS notification through Tachyon notification providers.

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**phone_number** | **str** | Destination phone number. E.164 format is recommended, for example +15551234567. |
**message** | **str** | Plain text SMS message body. |

## Example

```python
from tachyon_sdk.models.send_sms_notification_request import SendSmsNotificationRequest

# TODO update the JSON string below
json = "{}"
# create an instance of SendSmsNotificationRequest from a JSON string
send_sms_notification_request_instance = SendSmsNotificationRequest.from_json(json)
# print the JSON string representation of the object
print(SendSmsNotificationRequest.to_json())

# convert the object into a dict
send_sms_notification_request_dict = send_sms_notification_request_instance.to_dict()
# create an instance of SendSmsNotificationRequest from a dict
send_sms_notification_request_from_dict = SendSmsNotificationRequest.from_dict(send_sms_notification_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
