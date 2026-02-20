# ToolSchema

Schema type for tool parameters

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**properties** | **object** |  | 
**required** | **List[str]** |  | 
**type** | **str** |  | 

## Example

```python
from tachyon_sdk.models.tool_schema import ToolSchema

# TODO update the JSON string below
json = "{}"
# create an instance of ToolSchema from a JSON string
tool_schema_instance = ToolSchema.from_json(json)
# print the JSON string representation of the object
print(ToolSchema.to_json())

# convert the object into a dict
tool_schema_dict = tool_schema_instance.to_dict()
# create an instance of ToolSchema from a dict
tool_schema_from_dict = ToolSchema.from_dict(tool_schema_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


