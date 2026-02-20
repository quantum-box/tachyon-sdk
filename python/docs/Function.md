# Function

Function type for tool definitions

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**description** | **str** | A description of what the function does | 
**name** | **str** | The name of the function | 
**parameters** | [**ToolSchema**](ToolSchema.md) | The parameters the function accepts | 
**strict** | **bool** | Whether to enforce strict parameter validation | [optional] [default to False]

## Example

```python
from tachyon_sdk.models.function import Function

# TODO update the JSON string below
json = "{}"
# create an instance of Function from a JSON string
function_instance = Function.from_json(json)
# print the JSON string representation of the object
print(Function.to_json())

# convert the object into a dict
function_dict = function_instance.to_dict()
# create an instance of Function from a dict
function_from_dict = Function.from_dict(function_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


