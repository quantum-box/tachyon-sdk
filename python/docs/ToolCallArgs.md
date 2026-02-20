# ToolCallArgs

Represents tool call arguments

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**args** | **object** |  | 
**tool_id** | **str** |  | 

## Example

```python
from tachyon_sdk.models.tool_call_args import ToolCallArgs

# TODO update the JSON string below
json = "{}"
# create an instance of ToolCallArgs from a JSON string
tool_call_args_instance = ToolCallArgs.from_json(json)
# print the JSON string representation of the object
print(ToolCallArgs.to_json())

# convert the object into a dict
tool_call_args_dict = tool_call_args_instance.to_dict()
# create an instance of ToolCallArgs from a dict
tool_call_args_from_dict = ToolCallArgs.from_dict(tool_call_args_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


