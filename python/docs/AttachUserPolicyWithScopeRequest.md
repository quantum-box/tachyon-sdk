# AttachUserPolicyWithScopeRequest

Request to attach a policy with resource scope

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**policy_id** | **str** | Policy ID to attach | 
**resource_scope** | **str** | Resource scope in TRN format | 
**tenant_id** | **str** | Tenant ID (operator) | 
**user_id** | **str** | User ID | 

## Example

```python
from tachyon_sdk.models.attach_user_policy_with_scope_request import AttachUserPolicyWithScopeRequest

# TODO update the JSON string below
json = "{}"
# create an instance of AttachUserPolicyWithScopeRequest from a JSON string
attach_user_policy_with_scope_request_instance = AttachUserPolicyWithScopeRequest.from_json(json)
# print the JSON string representation of the object
print(AttachUserPolicyWithScopeRequest.to_json())

# convert the object into a dict
attach_user_policy_with_scope_request_dict = attach_user_policy_with_scope_request_instance.to_dict()
# create an instance of AttachUserPolicyWithScopeRequest from a dict
attach_user_policy_with_scope_request_from_dict = AttachUserPolicyWithScopeRequest.from_dict(attach_user_policy_with_scope_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


