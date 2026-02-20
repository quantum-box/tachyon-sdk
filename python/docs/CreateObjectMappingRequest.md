# CreateObjectMappingRequest

Request to create an object mapping

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**entity_id** | **str** | Internal entity ID | 
**object_name** | **str** | Object type name | 
**provider_name** | **str** | Provider name (e.g., \&quot;HubSpot\&quot;) | 
**provider_object_id** | **str** | External provider object ID | 

## Example

```python
from tachyon_sdk.models.create_object_mapping_request import CreateObjectMappingRequest

# TODO update the JSON string below
json = "{}"
# create an instance of CreateObjectMappingRequest from a JSON string
create_object_mapping_request_instance = CreateObjectMappingRequest.from_json(json)
# print the JSON string representation of the object
print(CreateObjectMappingRequest.to_json())

# convert the object into a dict
create_object_mapping_request_dict = create_object_mapping_request_instance.to_dict()
# create an instance of CreateObjectMappingRequest from a dict
create_object_mapping_request_from_dict = CreateObjectMappingRequest.from_dict(create_object_mapping_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


