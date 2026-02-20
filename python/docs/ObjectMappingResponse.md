# ObjectMappingResponse

Response for an object mapping

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**created_at** | **str** |  | 
**entity_id** | **str** |  | 
**object_name** | **str** |  | 
**provider_name** | **str** |  | 
**provider_primary_id** | **str** |  | 
**tenant_id** | **str** |  | 
**updated_at** | **str** |  | 

## Example

```python
from tachyon_sdk.models.object_mapping_response import ObjectMappingResponse

# TODO update the JSON string below
json = "{}"
# create an instance of ObjectMappingResponse from a JSON string
object_mapping_response_instance = ObjectMappingResponse.from_json(json)
# print the JSON string representation of the object
print(ObjectMappingResponse.to_json())

# convert the object into a dict
object_mapping_response_dict = object_mapping_response_instance.to_dict()
# create an instance of ObjectMappingResponse from a dict
object_mapping_response_from_dict = ObjectMappingResponse.from_dict(object_mapping_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


