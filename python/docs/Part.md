# Part


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**text** | **str** |  | 
**type** | **str** |  | 
**image** | **str** |  | 
**file** | **str** |  | 
**args** | **str** |  | 
**tool_call_id** | **str** |  | 
**tool_name** | **str** |  | 
**result** | **str** |  | 

## Example

```python
from tachyon_sdk.models.part import Part

# TODO update the JSON string below
json = "{}"
# create an instance of Part from a JSON string
part_instance = Part.from_json(json)
# print the JSON string representation of the object
print(Part.to_json())

# convert the object into a dict
part_dict = part_instance.to_dict()
# create an instance of Part from a dict
part_from_dict = Part.from_dict(part_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


