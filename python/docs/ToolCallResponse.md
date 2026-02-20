# ToolCallResponse

Tool call response type

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**function** | [**FunctionCallResponse**](FunctionCallResponse.md) | The function call details | 
**id** | **str** | The ID of the tool call | 
**type** | **str** | The type of the tool call | 

## Example

```python
from tachyon_sdk.models.tool_call_response import ToolCallResponse

# TODO update the JSON string below
json = "{}"
# create an instance of ToolCallResponse from a JSON string
tool_call_response_instance = ToolCallResponse.from_json(json)
# print the JSON string representation of the object
print(ToolCallResponse.to_json())

# convert the object into a dict
tool_call_response_dict = tool_call_response_instance.to_dict()
# create an instance of ToolCallResponse from a dict
tool_call_response_from_dict = ToolCallResponse.from_dict(tool_call_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


