# AddUserToTenantRequest

Request DTO for adding a user to a tenant

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**tenant_id** | **str** |  | 

## Example

```python
from tachyon_sdk.models.add_user_to_tenant_request import AddUserToTenantRequest

# TODO update the JSON string below
json = "{}"
# create an instance of AddUserToTenantRequest from a JSON string
add_user_to_tenant_request_instance = AddUserToTenantRequest.from_json(json)
# print the JSON string representation of the object
print(AddUserToTenantRequest.to_json())

# convert the object into a dict
add_user_to_tenant_request_dict = add_user_to_tenant_request_instance.to_dict()
# create an instance of AddUserToTenantRequest from a dict
add_user_to_tenant_request_from_dict = AddUserToTenantRequest.from_dict(add_user_to_tenant_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


