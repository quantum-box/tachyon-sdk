# UpdateUserRoleRequest

Request DTO for updating user role

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**role** | **str** |  | 
**tenant_id** | **str** |  | 

## Example

```python
from tachyon_sdk.models.update_user_role_request import UpdateUserRoleRequest

# TODO update the JSON string below
json = "{}"
# create an instance of UpdateUserRoleRequest from a JSON string
update_user_role_request_instance = UpdateUserRoleRequest.from_json(json)
# print the JSON string representation of the object
print(UpdateUserRoleRequest.to_json())

# convert the object into a dict
update_user_role_request_dict = update_user_role_request_instance.to_dict()
# create an instance of UpdateUserRoleRequest from a dict
update_user_role_request_from_dict = UpdateUserRoleRequest.from_dict(update_user_role_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


