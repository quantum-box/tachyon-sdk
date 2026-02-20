# Thinking

Thinking/reasoning content chunk

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**index** | **int** |  | 
**is_finished** | **bool** |  | 
**text** | **str** |  | 

## Example

```python
from tachyon_sdk.models.thinking import Thinking

# TODO update the JSON string below
json = "{}"
# create an instance of Thinking from a JSON string
thinking_instance = Thinking.from_json(json)
# print the JSON string representation of the object
print(Thinking.to_json())

# convert the object into a dict
thinking_dict = thinking_instance.to_dict()
# create an instance of Thinking from a dict
thinking_from_dict = Thinking.from_dict(thinking_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


