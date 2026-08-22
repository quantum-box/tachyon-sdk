# AllowedTenantResponse

Allowed tenant metadata. Denied tenants and policy internals are omitted.

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**alias** | **str** |  | 
**execution_mode** | **str** |  | 
**name** | **str** |  | 
**parent_tenant_id** | **str** |  | [optional] 
**root_tenant_id** | **str** |  | 
**tenant_id** | **str** |  | 

## Example

```python
from tachyon_sdk.models.allowed_tenant_response import AllowedTenantResponse

# TODO update the JSON string below
json = "{}"
# create an instance of AllowedTenantResponse from a JSON string
allowed_tenant_response_instance = AllowedTenantResponse.from_json(json)
# print the JSON string representation of the object
print(AllowedTenantResponse.to_json())

# convert the object into a dict
allowed_tenant_response_dict = allowed_tenant_response_instance.to_dict()
# create an instance of AllowedTenantResponse from a dict
allowed_tenant_response_from_dict = AllowedTenantResponse.from_dict(allowed_tenant_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


