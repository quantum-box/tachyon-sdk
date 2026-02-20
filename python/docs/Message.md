# Message

Message type for request/response handling

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**content** | [**MessageContent**](MessageContent.md) | The content of the message | [optional] 
**role** | **str** | The role of the message author | 
**tool_calls** | [**List[ToolCallResponse]**](ToolCallResponse.md) | The tool calls made by the assistant | [optional] 

## Example

```python
from tachyon_sdk.models.message import Message

# TODO update the JSON string below
json = "{}"
# create an instance of Message from a JSON string
message_instance = Message.from_json(json)
# print the JSON string representation of the object
print(Message.to_json())

# convert the object into a dict
message_dict = message_instance.to_dict()
# create an instance of Message from a dict
message_from_dict = Message.from_dict(message_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


