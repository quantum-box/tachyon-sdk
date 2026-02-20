# TenantMappingResponse

Tenant mapping info

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**provider_name** | **str** |  | 
**provider_tenant_id** | **str** |  | 
**tenant_id** | **str** |  | 

## Example

```python
from tachyon_sdk.models.tenant_mapping_response import TenantMappingResponse

# TODO update the JSON string below
json = "{}"
# create an instance of TenantMappingResponse from a JSON string
tenant_mapping_response_instance = TenantMappingResponse.from_json(json)
# print the JSON string representation of the object
print(TenantMappingResponse.to_json())

# convert the object into a dict
tenant_mapping_response_dict = tenant_mapping_response_instance.to_dict()
# create an instance of TenantMappingResponse from a dict
tenant_mapping_response_from_dict = TenantMappingResponse.from_dict(tenant_mapping_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


