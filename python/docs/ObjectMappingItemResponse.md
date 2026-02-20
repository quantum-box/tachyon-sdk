# ObjectMappingItemResponse

Single item in object mapping list

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**object_mapping** | [**ObjectMappingResponse**](ObjectMappingResponse.md) |  | 
**tenant_mapping** | [**TenantMappingResponse**](TenantMappingResponse.md) |  | [optional] 

## Example

```python
from tachyon_sdk.models.object_mapping_item_response import ObjectMappingItemResponse

# TODO update the JSON string below
json = "{}"
# create an instance of ObjectMappingItemResponse from a JSON string
object_mapping_item_response_instance = ObjectMappingItemResponse.from_json(json)
# print the JSON string representation of the object
print(ObjectMappingItemResponse.to_json())

# convert the object into a dict
object_mapping_item_response_dict = object_mapping_item_response_instance.to_dict()
# create an instance of ObjectMappingItemResponse from a dict
object_mapping_item_response_from_dict = ObjectMappingItemResponse.from_dict(object_mapping_item_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


