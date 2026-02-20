# ObjectMappingListResponse

Response for object mapping list

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**items** | [**List[ObjectMappingItemResponse]**](ObjectMappingItemResponse.md) |  | 

## Example

```python
from tachyon_sdk.models.object_mapping_list_response import ObjectMappingListResponse

# TODO update the JSON string below
json = "{}"
# create an instance of ObjectMappingListResponse from a JSON string
object_mapping_list_response_instance = ObjectMappingListResponse.from_json(json)
# print the JSON string representation of the object
print(ObjectMappingListResponse.to_json())

# convert the object into a dict
object_mapping_list_response_dict = object_mapping_list_response_instance.to_dict()
# create an instance of ObjectMappingListResponse from a dict
object_mapping_list_response_from_dict = ObjectMappingListResponse.from_dict(object_mapping_list_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


