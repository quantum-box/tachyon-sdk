# AttachUserPolicyRequest

Request to attach a policy to a user

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**policy_id** | **str** | Policy ID to attach | 
**tenant_id** | **str** | Tenant ID (operator) | 
**user_id** | **str** | User ID | 

## Example

```python
from tachyon_sdk.models.attach_user_policy_request import AttachUserPolicyRequest

# TODO update the JSON string below
json = "{}"
# create an instance of AttachUserPolicyRequest from a JSON string
attach_user_policy_request_instance = AttachUserPolicyRequest.from_json(json)
# print the JSON string representation of the object
print(AttachUserPolicyRequest.to_json())

# convert the object into a dict
attach_user_policy_request_dict = attach_user_policy_request_instance.to_dict()
# create an instance of AttachUserPolicyRequest from a dict
attach_user_policy_request_from_dict = AttachUserPolicyRequest.from_dict(attach_user_policy_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


