# DetachUserPolicyWithScopeRequest

Request to detach a policy with resource scope

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**policy_id** | **str** | Policy ID to detach | 
**resource_scope** | **str** | Resource scope in TRN format | 
**tenant_id** | **str** | Tenant ID (operator) | 
**user_id** | **str** | User ID | 

## Example

```python
from tachyon_sdk.models.detach_user_policy_with_scope_request import DetachUserPolicyWithScopeRequest

# TODO update the JSON string below
json = "{}"
# create an instance of DetachUserPolicyWithScopeRequest from a JSON string
detach_user_policy_with_scope_request_instance = DetachUserPolicyWithScopeRequest.from_json(json)
# print the JSON string representation of the object
print(DetachUserPolicyWithScopeRequest.to_json())

# convert the object into a dict
detach_user_policy_with_scope_request_dict = detach_user_policy_with_scope_request_instance.to_dict()
# create an instance of DetachUserPolicyWithScopeRequest from a dict
detach_user_policy_with_scope_request_from_dict = DetachUserPolicyWithScopeRequest.from_dict(detach_user_policy_with_scope_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


