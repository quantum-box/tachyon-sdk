# DeltaMessage


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**content** | **str** | The content of the message | [optional] 
**role** | [**ChatRole**](ChatRole.md) | The role of the message author | [optional] 
**tool_calls** | [**List[ToolCall]**](ToolCall.md) | The tool calls | [optional] 

## Example

```python
from tachyon_sdk.models.delta_message import DeltaMessage

# TODO update the JSON string below
json = "{}"
# create an instance of DeltaMessage from a JSON string
delta_message_instance = DeltaMessage.from_json(json)
# print the JSON string representation of the object
print(DeltaMessage.to_json())

# convert the object into a dict
delta_message_dict = delta_message_instance.to_dict()
# create an instance of DeltaMessage from a dict
delta_message_from_dict = DeltaMessage.from_dict(delta_message_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


