# ToolResult

Represents a tool execution result

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**is_finished** | **bool** |  | 
**result** | **str** |  | 
**tool_id** | **str** |  | 

## Example

```python
from tachyon_sdk.models.tool_result import ToolResult

# TODO update the JSON string below
json = "{}"
# create an instance of ToolResult from a JSON string
tool_result_instance = ToolResult.from_json(json)
# print the JSON string representation of the object
print(ToolResult.to_json())

# convert the object into a dict
tool_result_dict = tool_result_instance.to_dict()
# create an instance of ToolResult from a dict
tool_result_from_dict = ToolResult.from_dict(tool_result_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


