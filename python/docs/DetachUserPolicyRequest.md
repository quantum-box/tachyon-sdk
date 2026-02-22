# DetachUserPolicyRequest

Request to detach a policy from a user

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**policy_id** | **str** | Policy ID to detach | 
**tenant_id** | **str** | Tenant ID (operator) | 
**user_id** | **str** | User ID | 

## Example

```python
from tachyon_sdk.models.detach_user_policy_request import DetachUserPolicyRequest

# TODO update the JSON string below
json = "{}"
# create an instance of DetachUserPolicyRequest from a JSON string
detach_user_policy_request_instance = DetachUserPolicyRequest.from_json(json)
# print the JSON string representation of the object
print(DetachUserPolicyRequest.to_json())

# convert the object into a dict
detach_user_policy_request_dict = detach_user_policy_request_instance.to_dict()
# create an instance of DetachUserPolicyRequest from a dict
detach_user_policy_request_from_dict = DetachUserPolicyRequest.from_dict(detach_user_policy_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


