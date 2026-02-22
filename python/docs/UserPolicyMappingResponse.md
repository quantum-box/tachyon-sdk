# UserPolicyMappingResponse

Response for user policy mapping

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**assigned_at** | **str** |  | 
**policy_id** | **str** |  | 
**resource_scope** | **str** |  | [optional] 
**tenant_id** | **str** |  | 
**user_id** | **str** |  | 

## Example

```python
from tachyon_sdk.models.user_policy_mapping_response import UserPolicyMappingResponse

# TODO update the JSON string below
json = "{}"
# create an instance of UserPolicyMappingResponse from a JSON string
user_policy_mapping_response_instance = UserPolicyMappingResponse.from_json(json)
# print the JSON string representation of the object
print(UserPolicyMappingResponse.to_json())

# convert the object into a dict
user_policy_mapping_response_dict = user_policy_mapping_response_instance.to_dict()
# create an instance of UserPolicyMappingResponse from a dict
user_policy_mapping_response_from_dict = UserPolicyMappingResponse.from_dict(user_policy_mapping_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


