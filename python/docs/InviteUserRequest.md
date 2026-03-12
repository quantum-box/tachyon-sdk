# InviteUserRequest

Request DTO for inviting a user

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**invitee_email** | **str** | Email to invite (mutually exclusive with invitee_id) | [optional] 
**invitee_id** | **str** | User ID to invite (mutually exclusive with invitee_email) | [optional] 
**notify_user** | **bool** |  | [optional] 
**platform_id** | **str** |  | [optional] 
**tenant_id** | **str** |  | 

## Example

```python
from tachyon_sdk.models.invite_user_request import InviteUserRequest

# TODO update the JSON string below
json = "{}"
# create an instance of InviteUserRequest from a JSON string
invite_user_request_instance = InviteUserRequest.from_json(json)
# print the JSON string representation of the object
print(InviteUserRequest.to_json())

# convert the object into a dict
invite_user_request_dict = invite_user_request_instance.to_dict()
# create an instance of InviteUserRequest from a dict
invite_user_request_from_dict = InviteUserRequest.from_dict(invite_user_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


