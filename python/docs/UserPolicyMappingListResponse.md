# UserPolicyMappingListResponse

Response for user policy mappings list

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**mappings** | [**List[UserPolicyMappingResponse]**](UserPolicyMappingResponse.md) |  | 

## Example

```python
from tachyon_sdk.models.user_policy_mapping_list_response import UserPolicyMappingListResponse

# TODO update the JSON string below
json = "{}"
# create an instance of UserPolicyMappingListResponse from a JSON string
user_policy_mapping_list_response_instance = UserPolicyMappingListResponse.from_json(json)
# print the JSON string representation of the object
print(UserPolicyMappingListResponse.to_json())

# convert the object into a dict
user_policy_mapping_list_response_dict = user_policy_mapping_list_response_instance.to_dict()
# create an instance of UserPolicyMappingListResponse from a dict
user_policy_mapping_list_response_from_dict = UserPolicyMappingListResponse.from_dict(user_policy_mapping_list_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


